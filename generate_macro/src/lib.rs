use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Pair, FnArg, ItemTrait, ReturnType, TraitItem, TraitItemMethod,
};
use tonic_build::{Method, Service};

struct MyMethod {
    pub name: String,
    pub identifier: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub request: proc_macro2::TokenStream,
    pub response: proc_macro2::TokenStream,
}

impl Method for MyMethod {
    const CODEC_PATH: &'static str = "tonic_trivial::json_codec::MyCodec";
    type Comment = String;

    fn name(&self) -> &str {
        &self.name
    }
    fn identifier(&self) -> &str {
        &self.identifier
    }
    fn comment(&self) -> &[Self::Comment] {
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
        (self.request.clone(), self.response.clone())
    }
}

struct MyService {
    pub name: String,
    pub package: String,
    pub identifier: String,
    pub methods: Vec<MyMethod>,
}

impl Service for MyService {
    const CODEC_PATH: &'static str = "tonic_trivial::json_codec::MyCodec";
    type Comment = String;
    type Method = MyMethod;

    fn name(&self) -> &str {
        &self.name
    }
    fn package(&self) -> &str {
        &self.package
    }
    fn identifier(&self) -> &str {
        &self.identifier
    }
    fn comment(&self) -> &[Self::Comment] {
        &[]
    }
    fn methods(&self) -> &[Self::Method] {
        &self.methods
    }
}

#[proc_macro]
pub fn generate_code(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let method = MyMethod {
        name: "count".to_string(),
        identifier: "Count".to_string(),
        client_streaming: false,
        server_streaming: false,
        request: quote! { super::CountRequest },
        response: quote! { super::CountResponse },
    };
    let service = MyService {
        name: "Count".to_string(),
        package: "counter".to_string(),
        identifier: "Count".to_string(),
        methods: vec![method],
    };
    let client = tonic_build::client::generate(&service, "");
    let server = tonic_build::server::generate(&service, "");
    (quote! {
        #client
        #server
    })
    .into()
}

fn make_method(method: TraitItemMethod) -> MyMethod {
    let name = method.sig.ident.to_string();
    let mut args: Vec<_> = method.sig.inputs.into_pairs().collect();
    if args.len() != 1 {
        panic!("Invalid rpc argument type");
    }
    let request = match args.pop() {
        Some(Pair::End(FnArg::Typed(pat))) => pat.ty.into_token_stream(),
        _ => panic!("Invalid rpc argument type"),
    };
    let response = match method.sig.output {
        ReturnType::Default => quote! { "()" },
        ReturnType::Type(_arrow, ty) => ty.into_token_stream(),
    };
    MyMethod {
        identifier: name.clone(),
        name,
        client_streaming: false,
        server_streaming: false,
        request,
        response,
    }
}

#[proc_macro_attribute]
pub fn tonic_rpc(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    eprintln!("tokens: {}", item);
    let trait_ = parse_macro_input!(item as ItemTrait);
    eprintln!("trait: {:?}", trait_);
    let name = trait_.ident.to_string();
    let methods: Vec<_> = trait_
        .items
        .into_iter()
        .filter_map(|item| match item {
            TraitItem::Method(method) => Some(make_method(method)),
            _ => None,
        })
        .collect();
    let service = MyService {
        package: name.clone(),
        identifier: name.clone(),
        name,
        methods,
    };
    let client = tonic_build::client::generate(&service, "");
    let server = tonic_build::server::generate(&service, "");
    (quote! {
        #client
        #server
    })
    .into()
}
