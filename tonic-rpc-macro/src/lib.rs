use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Pair, FnArg, ItemTrait, ReturnType, TraitItem, TraitItemMethod,
    Type,
};
use tonic_build::{Attributes, Method, Service};

struct RustDefMethod {
    pub name: String,
    pub identifier: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub request: proc_macro2::TokenStream,
    pub response: proc_macro2::TokenStream,
    pub generated_request: syn::Ident,
    pub generated_response: syn::Ident,
    pub doc_comments: Vec<String>,
}

impl RustDefMethod {
    fn name(&self) -> &str {
        &self.name
    }
    fn identifier(&self) -> &str {
        &self.identifier
    }
    fn comment(&self) -> &[String] {
        &self.doc_comments
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
                _compile_well_known_types: bool,
            ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
                self.0.request_response_name(s)
            }
        }
    };
}

method_impl!(JsonMethod, "::tonic_rpc::codec::JsonCodec");
method_impl!(BincodeMethod, "::tonic_rpc::codec::BincodeCodec");
method_impl!(CborMethod, "::tonic_rpc::codec::CborCodec");
method_impl!(MessagePackMethod, "::tonic_rpc::codec::MessagePackCodec");

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

service_impl!(JsonMethod, "::tonic_rpc::codec::JsonCodec");
service_impl!(BincodeMethod, "::tonic_rpc::codec::BincodeCodec");
service_impl!(CborMethod, "::tonic_rpc::codec::CborCodec");
service_impl!(MessagePackMethod, "::tonic_rpc::codec::MessagePackCodec");

/// Return value is `(server_streaming, client_streaming, doc_comments)`.
fn parse_attributes(attributes: Vec<syn::Attribute>) -> (bool, bool, Vec<String>) {
    let mut server_streaming = false;
    let mut client_streaming = false;
    let mut doc_comments = Vec::new();

    for attr in attributes {
        if attr.path.is_ident("server_streaming") {
            server_streaming = true;
        } else if attr.path.is_ident("client_streaming") {
            client_streaming = true;
        } else if attr.path.is_ident("doc") {
            if let Some(comment) = attr
                .tokens
                .to_string()
                .strip_prefix("= \"")
                .and_then(|c| c.strip_suffix('\"'))
            {
                doc_comments.push(comment.to_string())
            }
        } else {
            panic!("Attribute {:?} is not supported on tonic-rpc methods", attr)
        }
    }

    (server_streaming, client_streaming, doc_comments)
}

fn make_method<T: From<RustDefMethod>>(method: TraitItemMethod, trait_name: &str) -> T {
    fn extract_arg<P>(arg: Pair<FnArg, P>) -> Box<Type> {
        match arg {
            Pair::Punctuated(FnArg::Typed(pat), _) | Pair::End(FnArg::Typed(pat)) => pat.ty,
            Pair::Punctuated(FnArg::Receiver(rec), _) | Pair::End(FnArg::Receiver(rec)) => panic!(
                "Invalid RPC argument. 'self' arguments are not allowed: {}",
                rec.to_token_stream()
            ),
        }
    }

    let name = method.sig.ident.to_string();
    let (server_streaming, client_streaming, doc_comments) = parse_attributes(method.attrs);

    let args: Vec<_> = method.sig.inputs.into_pairs().map(extract_arg).collect();
    let request = match args.len() {
        1 => args[0].to_token_stream(),
        _ => {
            let tuple_fields: proc_macro2::TokenStream = itertools::Itertools::intersperse(
                args.into_iter().map(|t| t.to_token_stream()),
                quote! {,},
            )
            .collect();
            quote! { ( #tuple_fields )}
        }
    };
    let response = match method.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_arrow, ty) => ty.to_token_stream(),
    };
    let generated_request =
        quote::format_ident!("__tonic_generated_{}_{}_request", trait_name, name);
    let generated_response =
        quote::format_ident!("__tonic_generated_{}_{}_response", trait_name, name);

    RustDefMethod {
        identifier: heck::CamelCase::to_camel_case(name.as_str()),
        name,
        client_streaming,
        server_streaming,
        request,
        response,
        generated_request,
        generated_response,
        doc_comments,
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
        package: "".to_string(),
        identifier: name.clone(),
        name,
        methods,
    };
    let client = tonic_build::client::generate(&service, false, "", false, &Attributes::default());
    let server = tonic_build::server::generate(&service, false, "", false, &Attributes::default());
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
