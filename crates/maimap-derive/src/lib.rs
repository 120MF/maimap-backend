use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(ToResponse, attributes(DoNotRespond))]
pub fn derive_to_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = match &input.data {
        Data::Struct(data) => {
            let fields = match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => {
                    return quote! {
                        compile_error!("ToResponse只支持命名字段的结构体");
                    }
                    .into();
                }
            };

            let field_conversions = fields.iter().filter_map(|field| {
                // 检查是否有DoNotRespond属性
                if field.attrs.iter().any(|attr|
                    attr.path().is_ident("DoNotRespond")) {
                    return None;
                }

                let field_name = field.ident.as_ref().unwrap();
                let field_ident = format_ident!("{}", field_name);
                let field_name_str = field_name.to_string();

                // 特殊处理_id字段
                if field_name_str == "_id" {
                    return Some(quote! {
                        map.insert("id".to_string(), serde_json::json!(self.#field_ident.to_string()));
                    });
                }

                // 根据类型生成转换代码
                let ty = &field.ty;
                let is_datetime = is_type_match(ty, "DateTime");
                let is_objectid = is_type_match(ty, "ObjectId");
                let is_decimal128 = is_type_match(ty, "Decimal128");

                let conversion = if is_datetime {
                    quote! {
                        map.insert(#field_name_str.to_string(),
                            serde_json::json!(self.#field_ident.try_to_rfc3339_string().unwrap_or_default()));
                    }
                } else if is_objectid {
                    quote! {
                        map.insert(#field_name_str.to_string(),
                            serde_json::json!(self.#field_ident.to_string()));
                    }
                } else if is_decimal128 {
                    quote! {
                        map.insert(#field_name_str.to_string(),
                            serde_json::json!(self.#field_ident.to_string().parse::<f64>().unwrap_or(0.0)));
                    }
                } else {
                    quote! {
                        map.insert(#field_name_str.to_string(),
                            serde_json::to_value(&self.#field_ident).unwrap_or(serde_json::Value::Null));
                    }
                };

                Some(conversion)
            }).collect::<Vec<_>>();

            quote! {
                impl ToResponse for #name {
                    fn to_response(&self) -> serde_json::Value {
                        let mut map = serde_json::Map::new();

                        #(#field_conversions)*

                        serde_json::Value::Object(map)
                    }
                }
            }
        }
        _ => quote! {
            compile_error!("ToResponse只能用于结构体");
        },
    };

    TokenStream::from(expanded)
}

// 辅助函数: 检查类型是否匹配指定名称
fn is_type_match(ty: &Type, type_name: &str) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == type_name;
        }
    }
    false
}
