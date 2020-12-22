use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Pair, FnArg, ItemTrait, ReturnType, TraitItem, TraitItemMethod,
};
use tonic_build::{Method, Service};

struct RustDefMethod {
    pub name: String,
    pub identifier: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub request: proc_macro2::TokenStream,
    pub response: proc_macro2::TokenStream,
    pub generated_request: syn::Ident,
    pub generated_response: syn::Ident,
}

impl RustDefMethod {
    fn name(&self) -> &str {
        &self.name
    }
    fn identifier(&self) -> &str {
        &self.identifier
    }
    fn comment(&self) -> &[String] {
        &[]
    }
    fn client_streaming(&self) -> bool {
        self.client_streaming
    }
    fn server_streaming(&self) -> bool {
        self.server_streaming
    }
    fn request_response_name(
        &self,
        _: &str,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let request = self.generated_request.clone();
        let response = self.generated_response.clone();
        (quote! {super::#request}, quote! {super::#response})
    }
}

trait RequestResponseTypes {
    fn generated_request(&self) -> &proc_macro2::Ident;
    fn generated_response(&self) -> &proc_macro2::Ident;
    fn request(&self) -> &proc_macro2::TokenStream;
    fn response(&self) -> &proc_macro2::TokenStream;
}

macro_rules! method_impl {
    ($name:ident, $codec:expr) => {
        struct $name(RustDefMethod);

        impl From<RustDefMethod> for $name {
            fn from(inner: RustDefMethod) -> Self {
                $name(inner)
            }
        }

        impl RequestResponseTypes for $name {
            fn generated_request(&self) -> &proc_macro2::Ident {
                &self.0.generated_request
            }
            fn generated_response(&self) -> &proc_macro2::Ident {
                &self.0.generated_response
            }
            fn request(&self) -> &proc_macro2::TokenStream {
                &self.0.request
            }
            fn response(&self) -> &proc_macro2::TokenStream {
                &self.0.response
            }
        }

        impl Method for $name {
            const CODEC_PATH: &'static str = $codec;
            type Comment = String;

            fn name(&self) -> &str {
                self.0.name()
            }
            fn identifier(&self) -> &str {
                self.0.identifier()
            }
            fn comment(&self) -> &[Self::Comment] {
                self.0.comment()
            }
            fn client_streaming(&self) -> bool {
                self.0.client_streaming()
            }
            fn server_streaming(&self) -> bool {
                self.0.server_streaming()
            }
            fn request_response_name(
                &self,
                s: &str,
            ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
                self.0.request_response_name(s)
            }
        }
    };
}

method_impl!(JsonMethod, "tonic_rpc::codec::JsonCodec");
method_impl!(BincodeMethod, "tonic_rpc::codec::BincodeCodec");
method_impl!(CborMethod, "tonic_rpc::codec::CborCodec");
method_impl!(MessagePackMethod, "tonic_rpc::codec::MessagePackCodec");

struct RustDefService<T> {
    pub name: String,
    pub package: String,
    pub identifier: String,
    pub methods: Vec<T>,
}

macro_rules! service_impl {
    ($name:ident, $codec:expr) => {
        impl Service for RustDefService<$name> {
            const CODEC_PATH: &'static str = $codec;
            type Comment = String;
            type Method = $name;

            fn name(&self) -> &str {
                &self.name
            }
            fn package(&self) -> &str {
                &self.package
            }
            fn identifier(&self) -> &str {
                &self.identifier
            }
            fn comment(&self) -> &[String] {
                &[]
            }
            fn methods(&self) -> &[Self::Method] {
                &self.methods
            }
        }
    };
}

service_impl!(JsonMethod, "tonic_rpc::codec::JsonCodec");
service_impl!(BincodeMethod, "tonic_rpc::codec::BincodeCodec");
service_impl!(CborMethod, "tonic_rpc::codec::CborCodec");
service_impl!(MessagePackMethod, "tonic_rpc::codec::MessagePackCodec");

fn make_method<T: From<RustDefMethod>>(method: TraitItemMethod, trait_name: &str) -> T {
    let name = method.sig.ident.to_string();
    let server_streaming = method
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("server_streaming"));
    let client_streaming = method
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("client_streaming"));
    let mut args: Vec<_> = method.sig.inputs.into_pairs().collect();
    if args.len() != 1 {
        panic!("Invalid rpc argument type");
    }
    let request = match args.pop() {
        Some(Pair::End(FnArg::Typed(pat))) => pat.ty.to_token_stream(),
        _ => panic!("Invalid rpc argument type"),
    };
    let response = match method.sig.output {
        ReturnType::Default => quote! { "()" },
        ReturnType::Type(_arrow, ty) => ty.to_token_stream(),
    };
    RustDefMethod {
        identifier: name.clone(),
        name: name.clone(),
        client_streaming,
        server_streaming,
        request,
        response,
        generated_request: quote::format_ident!(
            "__tonic_generated_{}_{}_request",
            trait_name,
            name.clone()
        ),
        generated_response: quote::format_ident!(
            "__tonic_generated_{}_{}_response",
            trait_name,
            name.clone()
        ),
    }
    .into()
}

fn make_rpc<T>(item: TokenStream) -> TokenStream
where
    T: From<RustDefMethod> + RequestResponseTypes,
    RustDefService<T>: Service,
{
    let trait_ = parse_macro_input!(item as ItemTrait);
    let name = trait_.ident.to_string();
    let methods: Vec<_> = trait_
        .items
        .into_iter()
        .filter_map(|item| match item {
            TraitItem::Method(method) => Some(make_method::<T>(method, &name)),
            _ => None,
        })
        .collect();
    let service = RustDefService {
        package: name.clone(),
        identifier: name.clone(),
        name,
        methods,
    };
    let client = tonic_build::client::generate(&service, "");
    let server = tonic_build::server::generate(&service, "");
    let types = service.methods.iter().map(|m| {
        let request_name = m.generated_request();
        let response_name = m.generated_response();
        let request_type = m.request();
        let response_type = m.response();
        quote! {
            type #request_name = #request_type;
            type #response_name = #response_type;
        }
    });
    let types = quote! { #( #types )*};
    (quote! {
        #types
        #client
        #server
    })
    .into()
}

#[proc_macro_attribute]
pub fn tonic_rpc(attributes: TokenStream, item: TokenStream) -> TokenStream {
    match attributes.to_string().as_str() {
        "json" => make_rpc::<JsonMethod>(item),
        "bincode" => make_rpc::<BincodeMethod>(item),
        "cbor" => make_rpc::<CborMethod>(item),
        "messagepack" => make_rpc::<MessagePackMethod>(item),
        "" => panic!("No tonic_rpc codec given"),
        other => panic!("Unrecognized tonic_rpc codec {}", other),
    }
}
