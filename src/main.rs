use serde::__private::size_hint;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

fn main() {
    println!("Please run the tests!");
}

#[derive(Debug, Eq, PartialEq)]
enum DictionaryOrSequence<T> {
    Sequence(Vec<T>),
    Dictionary(HashMap<String, T>),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for DictionaryOrSequence<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DictionaryOrSequenceVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for DictionaryOrSequenceVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = DictionaryOrSequence<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence or an object")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(size_hint::cautious(seq.size_hint()));

                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }

                Ok(DictionaryOrSequence::Sequence(values))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = HashMap::<String, T>::new();
                while let Some((key, value)) = map.next_entry::<String, T>()? {
                    values.insert(key, value);
                }

                Ok(DictionaryOrSequence::Dictionary(values))
            }
        }

        let visitor = DictionaryOrSequenceVisitor {
            marker: PhantomData,
        };

        deserializer.deserialize_any(visitor)
    }
}

#[cfg(test)]
mod test {
    use crate::DictionaryOrSequence;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[test]
    fn test() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct Example {
            something: DictionaryOrSequence<OtherExample>,
        }

        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct OtherExample {
            foo: String,
            bar: i32,
        }

        let data = r#"{ "something": { "david": { "foo": "hello", "bar": 23 }} }"#;
        let example: Example = serde_json::from_str(data).unwrap();

        assert_eq!(
            example,
            Example {
                something: DictionaryOrSequence::Dictionary(HashMap::from([(
                    String::from("david"),
                    OtherExample {
                        foo: String::from("hello"),
                        bar: 23
                    }
                )]))
            }
        );

        let data = r#"{ "something": [{ "foo": "hello", "bar": 23 }] }"#;
        let example: Example = serde_json::from_str(data).unwrap();

        assert_eq!(
            example,
            Example {
                something: DictionaryOrSequence::Sequence(vec![OtherExample {
                    foo: String::from("hello"),
                    bar: 23
                }])
            }
        );
    }
}
