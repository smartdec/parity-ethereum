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

pub fn impl_decodable(ast: &syn::DeriveInput) -> TokenStream {
	let expr = impl_decodable_inner(ast);
	let name = &ast.ident;
	let dummy_const = syn::Ident::new(&format!("_IMPL_RLP_DECODABLE_FOR_{}", name), Span::call_site());

	quote! {
		#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
		const #dummy_const: () = {
			extern crate rlp;
			impl rlp::Decodable for #name {
				fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
					Ok(#expr)
				}
			}
		};
	}
}

fn impl_decodable_inner(ast: &syn::DeriveInput) -> TokenStream {
	let name = &ast.ident;
	match ast.data {
		syn::Data::Struct(ref s) => {
			let stmts = s.fields.iter()
				.enumerate()
				.map(|(ind, f)| decodable_field(ind, f, decodable_parse_quotes()));

			quote! {
				#name {
					#(#stmts)*
				}
			}
		},
		syn::Data::Enum(ref en) => {
			let stmts = en.variants.iter()
				.map(|v| decodable_variant(name, v));
			quote! {
				{
					let variant_name: String = rlp.val_at(0)?;
					match variant_name.as_str() {
						#(#stmts)*
						_ => return Err(rlp::DecoderError::Custom("Wrong variant name")),
					}
				}
			}
		},
		_ => panic!("#[derive(RlpDecodable)] only supports structs and enums"),
	}
}

fn decodable_variant(enum_ident: &syn::Ident, variant: &syn::Variant) -> TokenStream {
	let ident = &variant.ident;
	let constr = match variant.fields {
		syn::Fields::Named(ref fields) => {
			let stmts = fields.named.iter()
				.enumerate()
				.map(|(ind, f)| decodable_field(ind, f, decodable_parse_quotes()));
			quote! {
				{
					let rlp = rlp.at(1)?;
					#enum_ident::#ident {
						#(#stmts)*
					}
				}
			}
		},
		syn::Fields::Unnamed(ref fields) => {
			let stmts = fields.unnamed.iter()
				.enumerate()
				.map(|(ind, f)| decodable_field(ind, f, decodable_parse_quotes()));
			quote! {
				{
					let rlp = rlp.at(1)?;
					#enum_ident::#ident {
						#(#stmts)*
					}
				}
			}
		},
		syn::Fields::Unit => quote! {
			#enum_ident::#ident
		},
	};
	quote! {
		stringify!(#ident) => #constr,
	}
}

struct ParseQuotes {
	single: TokenStream,
	list: TokenStream,
	takes_index: bool,
}

fn decodable_parse_quotes() -> ParseQuotes {
	ParseQuotes {
		single: quote! { rlp.val_at },
		list: quote! { rlp.list_at },
		takes_index: true,
	}
}

fn decodable_wrapper_parse_quotes() -> ParseQuotes {
	ParseQuotes {
		single: quote! { rlp.as_val },
		list: quote! { rlp.as_list },
		takes_index: false,
	}
}

fn decodable_field(index: usize, field: &syn::Field, quotes: ParseQuotes) -> TokenStream {
	let id = match field.ident {
		Some(ref ident) => quote! { #ident },
		None => {
			let index: syn::Index = index.into();
			quote! { #index }
		}
	};

	let index = quote! { #index };

	let single = quotes.single;
	let list = quotes.list;

	match field.ty {
		syn::Type::Path(ref path) => {
			let ident = &path.path.segments.first().expect("there must be at least 1 segment").value().ident;
			if &ident.to_string() == "Vec" {
				if quotes.takes_index {
					quote! { #id: #list(#index)?, }
				} else {
					quote! { #id: #list()?, }
				}
			} else {
				if quotes.takes_index {
					quote! { #id: #single(#index)?, }
				} else {
					quote! { #id: #single()?, }
				}
			}
		},
		_ => panic!("rlp_derive not supported"),
	}
}



pub fn impl_decodable_wrapper(ast: &syn::DeriveInput) -> TokenStream {
	let body = match ast.data {
		syn::Data::Struct(ref s) => s,
		_ => panic!("#[derive(RlpDecodableWrapper)] is only defined for structs."),
	};

	let stmt = {
		let fields: Vec<_> = body.fields.iter().collect();
		if fields.len() == 1 {
			let field = fields.first().expect("fields.len() == 1; qed");
			decodable_field(0, field, decodable_wrapper_parse_quotes())
		} else {
			panic!("#[derive(RlpEncodableWrapper)] is only defined for structs with one field.")
		}
	};

	let name = &ast.ident;

	let dummy_const = syn::Ident::new(&format!("_IMPL_RLP_DECODABLE_FOR_{}", name), Span::call_site());
	let impl_block = quote! {
		impl rlp::Decodable for #name {
			fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
				let result = #name {
					#stmt
				};

				Ok(result)
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
