use serde::de::{DeserializeSeed, MapAccess};

use de::{Deserializer, Result};
use error::{Error, KbinErrorKind};
use node_types::StandardType;

pub struct Map<'a, 'de: 'a> {
  de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> Map<'a, 'de> {
  pub fn new(de: &'a mut Deserializer<'de>) -> Self {
    Self { de }
  }
}

// TODO: FIX THIS, it's pretty broken, but it will deserialize correctly
impl<'de, 'a> MapAccess<'de> for Map<'a, 'de> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where K: DeserializeSeed<'de>
  {
    trace!("--> <Map as MapAccess>::next_key_seed()");

    let (node_type, _is_array) = self.de.reader.read_node_type()?;
    debug!("<Map as MapAccess>::next_key_seed() => node_type: {:?}", node_type);

    if node_type == StandardType::NodeEnd ||
       node_type == StandardType::FileEnd
    {
      trace!("<Map as MapAccess>::next_key_seed() => end of map");
      return Ok(None);
    }

    let key = seed.deserialize(&mut *self.de).map(Some)?;

    self.de.node_stack.push(node_type);

    Ok(key)
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where V: DeserializeSeed<'de>
  {
    debug!("--> <Map as MapAccess>::next_value_seed()");
    let value = seed.deserialize(&mut *self.de)?;

    let popped = self.de.node_stack.pop();
    debug!("<Map as MapAccess>::next_value_seed() => popped: {:?}, node_stack: {:?}", popped, self.de.node_stack);

    // Consume the NodeEnd
    match popped {
      Some(StandardType::Attribute) |
      Some(StandardType::NodeStart) => {},
      Some(_) => {
        let (node_type, _is_array) = self.de.reader.read_node_type()?;
        if node_type != StandardType::NodeEnd {
          return Err(KbinErrorKind::TypeMismatch(*StandardType::NodeEnd, *node_type).into());
        }
      },
      None => {},
    }

    Ok(value)
  }
}
