use linked_data::{
	xsd_types, CowRdfTerm, LinkedData, LinkedDataGraph, LinkedDataPredicateObjects,
	LinkedDataResource, LinkedDataSubject, RdfLiteral, RdfLiteralRef, ResourceInterpretation,
};
use locspan::Meta;
use rdf_types::{Interpretation, LanguageTagVocabularyMut, Term, Vocabulary};

use crate::{
	object::Literal,
	rdf::{XSD_DOUBLE, XSD_INTEGER},
	Value,
};

impl<M: Clone, V: Vocabulary, I: Interpretation> LinkedDataResource<V, I> for Value<V::Iri, M>
where
	V: LanguageTagVocabularyMut,
{
	fn interpretation(
		&self,
		vocabulary: &mut V,
		_interpretation: &mut I,
	) -> ResourceInterpretation<V, I> {
		let term = match self {
			Self::Literal(l, ty) => match l {
				Literal::Null => CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(
					xsd_types::Value::String("null".to_string()),
				))),
				Literal::Boolean(b) => CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(
					xsd_types::Value::Boolean(*b),
				))),
				Literal::Number(n) => {
					#[derive(Clone, Copy, Default, PartialEq)]
					enum NumericType {
						Integer,
						Double,
						#[default]
						Unknown,
					}

					impl NumericType {
						pub fn matches(self, other: Self) -> bool {
							self == other || self == Self::Unknown
						}
					}

					let ty = ty
						.as_ref()
						.and_then(|t| vocabulary.iri(t))
						.map(|iri| {
							if iri == XSD_INTEGER {
								NumericType::Integer
							} else if iri == XSD_DOUBLE {
								NumericType::Double
							} else {
								NumericType::Unknown
							}
						})
						.unwrap_or_default();

					let value = match n.as_i64() {
						Some(i) if ty.matches(NumericType::Integer) => {
							xsd_types::Value::Integer(i.into())
						}
						_ => xsd_types::Value::Double(n.as_f64_lossy().into()),
					};

					CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(value)))
				}
				Literal::String(s) => CowRdfTerm::Borrowed(Term::Literal(RdfLiteralRef::Xsd(
					xsd_types::ValueRef::String(s),
				))),
			},
			Self::LangString(s) => match s.language().and_then(|l| l.as_language_tag()) {
				Some(tag) => {
					let tag = vocabulary.insert_language_tag(tag);
					CowRdfTerm::Owned(Term::Literal(RdfLiteral::Any(
						s.as_str().to_owned(),
						rdf_types::literal::Type::LangString(tag),
					)))
				}
				None => CowRdfTerm::Borrowed(Term::Literal(RdfLiteralRef::Xsd(
					xsd_types::ValueRef::String(s.as_str()),
				))),
			},
			Self::Json(Meta(json, _)) => {
				let json = json.clone().map_metadata(|_| ());
				CowRdfTerm::Owned(Term::Literal(RdfLiteral::Json(json)))
			}
		};

		ResourceInterpretation::Uninterpreted(Some(term))
	}
}

impl<T, M, V: Vocabulary, I: Interpretation> LinkedDataSubject<V, I> for Value<T, M> {
	fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<V, I>,
	{
		visitor.end()
	}
}

impl<T, M, V: Vocabulary, I: Interpretation> LinkedDataPredicateObjects<V, I> for Value<T, M> {
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<V, I>,
	{
		visitor.end()
	}
}

impl<M: Clone, V: Vocabulary, I: Interpretation> LinkedDataGraph<V, I> for Value<V::Iri, M>
where
	V: LanguageTagVocabularyMut,
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<V, I>,
	{
		visitor.subject(self)?;
		visitor.end()
	}
}

impl<M: Clone, V: Vocabulary, I: Interpretation> LinkedData<V, I> for Value<V::Iri, M>
where
	V: LanguageTagVocabularyMut,
{
	fn visit<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<V, I>,
	{
		visitor.default_graph(self)?;
		visitor.end()
	}
}
