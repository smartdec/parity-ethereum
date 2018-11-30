// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use syn;
use proc_macro2::{TokenStream, Span};

pub fn impl_encodable(ast: &syn::DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let dummy_const = syn::Ident::new(&format!("_IMPL_RLP_ENCODABLE_FOR_{}", name), Span::call_site());
	let stmt = impl_encodable_inner(ast);
	quote! {
		#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
		const #dummy_const: () = {
			extern crate rlp;
			impl rlp::Encodable for #name {
				fn rlp_append(&self, stream: &mut rlp::RlpStream) {
					#stmt
				}
			}
		};
	}
}

fn impl_encodable_inner(ast: &syn::DeriveInput) -> TokenStream {
	match ast.data {
		syn::Data::Struct(ref s) => {
			let stmts: Vec<_> = s.fields.iter()
				.enumerate()
				.map(|(ind, f)| encodable_field(ind, f))
				.collect();
			let stmts_len = stmts.len();
			let stmts_len = quote! { #stmts_len };

			quote! {
				stream.begin_list(#stmts_len);
				#(#stmts)*
			}
		},
		syn::Data::Enum(ref en) => {
			let stmts = en.variants.iter()
				.map(|v| encodable_variant(&ast.ident, v));
			quote! {
				stream.begin_list(2);
				match self {
					#(#stmts)*
				}
			}
		},
		_ => panic!("#[derive(RlpEncodable)] only supports structs and enums"),
	}
}

fn encodable_variant(enum_ident: &syn::Ident, variant: &syn::Variant) -> TokenStream {
	let ident = &variant.ident;
	let pattern = match variant.fields {
		syn::Fields::Named(ref fields) => {
			let fields = fields.named.iter()
				.map(|f| f.ident.as_ref().unwrap());
			quote! {
				{ #(#fields),* }
			}
		},
		syn::Fields::Unnamed(ref fields) => {
			let fields = fields.unnamed.iter()
				.enumerate()
				.map(|(ind, _f)| syn::Ident::new(&format!("_f{}", ind), Span::call_site()));
			quote! {
				( #(#fields),* )
			}
		},
		syn::Fields::Unit => quote! {},
	};
	let appenders: Vec<TokenStream> = match variant.fields {
		syn::Fields::Named(ref fields) => {
			fields.named.iter()
				.map(|f| {
					let ident = f.ident.as_ref().unwrap();
					let ident = quote! { *#ident };
					encodable_field_inner(ident, f)
				})
				.collect()
		},
		syn::Fields::Unnamed(ref fields) => {
			fields.unnamed.iter()
				.enumerate()
				.map(|(ind, f)| {
					let ident = syn::Ident::new(&format!("_f{}", ind), Span::call_site());
					let ident = quote! { *#ident };
					encodable_field_inner(ident, f)
				})
				.collect()
		},
		syn::Fields::Unit => vec![],
	};
	let appenders_len = appenders.len();
	let appenders_len = quote! { #appenders_len };
	quote! {
		#enum_ident::#ident #pattern => {
			stream.append(&stringify!(#ident));
			stream.begin_list(#appenders_len);
			#(#appenders)*
		},
	}
}

fn encodable_field(index: usize, field: &syn::Field) -> TokenStream {
	let ident = match field.ident {
		Some(ref ident) => quote! { #ident },
		None => {
			let index: syn::Index = index.into();
			quote! { #index }
		}
	};

	let id = quote! { self.#ident };

	encodable_field_inner(id, field)
}

fn encodable_field_inner(id: TokenStream, field: &syn::Field) -> TokenStream {
	match field.ty {
		syn::Type::Path(ref path) => {
			let top_segment = path.path.segments.first().expect("there must be at least 1 segment");
			let ident = &top_segment.value().ident;
			if &ident.to_string() == "Vec" {
				let inner_ident = match top_segment.value().arguments {
					syn::PathArguments::AngleBracketed(ref angle) => {
						let ty = angle.args.first().expect("Vec has only one angle bracketed type; qed");
						match **ty.value() {
							syn::GenericArgument::Type(syn::Type::Path(ref path)) => &path.path.segments.first().expect("there must be at least 1 segment").value().ident,
							_ => panic!("rlp_derive not supported"),
						}
					},
					_ => unreachable!("Vec has only one angle bracketed type; qed"),
				};
				quote! { stream.append_list::<#inner_ident, _>(&#id); }
			} else {
				quote! { stream.append(&#id); }
			}
		},
		_ => panic!("rlp_derive not supported"),
	}
}



pub fn impl_encodable_wrapper(ast: &syn::DeriveInput) -> TokenStream {
	let body = match ast.data {
		syn::Data::Struct(ref s) => s,
		_ => panic!("#[derive(RlpEncodableWrapper)] is only defined for structs."),
	};

	let stmt = {
		let fields: Vec<_> = body.fields.iter().collect();
		if fields.len() == 1 {
			let field = fields.first().expect("fields.len() == 1; qed");
			encodable_field(0, field)
		} else {
			panic!("#[derive(RlpEncodableWrapper)] is only defined for structs with one field.")
		}
	};

	let name = &ast.ident;

	let dummy_const = syn::Ident::new(&format!("_IMPL_RLP_ENCODABLE_FOR_{}", name), Span::call_site());
	let impl_block = quote! {
		impl rlp::Encodable for #name {
			fn rlp_append(&self, stream: &mut rlp::RlpStream) {
				#stmt
			}
		}
	};

	quote! {
		#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
		const #dummy_const: () = {
			extern crate rlp;
			#impl_block
		};
	}
}
