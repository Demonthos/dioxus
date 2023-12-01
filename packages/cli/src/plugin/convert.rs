use super::interface::plugins::main::toml::*;
use toml as ext_toml;

#[async_trait]
pub trait ConvertWithState<T> {
    async fn convert_with_state(self, state: &mut PluginState) -> T;
}

pub trait Convert<T> {
    fn convert(self) -> T;
}

impl<T, U> Convert<Option<T>> for Option<U>
where
    U: Convert<T>,
{
    fn convert(self) -> Option<T> {
        self.map(Convert::convert)
    }
}

// Could these converts be simplified with bytemuck POD perhaps?
impl Convert<ext_toml::value::Datetime> for Datetime {
    fn convert(self) -> ext_toml::value::Datetime {
        let Datetime { date, time, offset } = self;
        ext_toml::value::Datetime {
            date: date.convert(),
            time: time.convert(),
            offset: offset.convert(),
        }
    }
}
impl Convert<Datetime> for ext_toml::value::Datetime {
    fn convert(self) -> Datetime {
        let ext_toml::value::Datetime { date, time, offset } = self;
        Datetime {
            date: date.convert(),
            time: time.convert(),
            offset: offset.convert(),
        }
    }
}

impl Convert<ext_toml::value::Time> for Time {
    fn convert(self) -> ext_toml::value::Time {
        let Time {
            hour,
            minute,
            second,
            nanosecond,
        } = self;
        ext_toml::value::Time {
            hour,
            minute,
            second,
            nanosecond,
        }
    }
}
impl Convert<Time> for ext_toml::value::Time {
    fn convert(self) -> Time {
        let ext_toml::value::Time {
            hour,
            minute,
            second,
            nanosecond,
        } = self;
        Time {
            hour,
            minute,
            second,
            nanosecond,
        }
    }
}

impl Convert<ext_toml::value::Date> for Date {
    fn convert(self) -> ext_toml::value::Date {
        let Date { year, month, day } = self;
        ext_toml::value::Date { year, month, day }
    }
}

impl Convert<ext_toml::value::Offset> for Offset {
    fn convert(self) -> ext_toml::value::Offset {
        match self {
            Offset::Z => ext_toml::value::Offset::Z,
            Offset::Custom((hours, minutes)) => ext_toml::value::Offset::Custom { hours, minutes },
        }
    }
}
impl Convert<Date> for ext_toml::value::Date {
    fn convert(self) -> Date {
        let ext_toml::value::Date { year, month, day } = self;
        Date { year, month, day }
    }
}

impl Convert<Offset> for ext_toml::value::Offset {
    fn convert(self) -> Offset {
        match self {
            ext_toml::value::Offset::Z => Offset::Z,
            ext_toml::value::Offset::Custom { hours, minutes } => Offset::Custom((hours, minutes)),
        }
    }
}
use async_trait::async_trait;
use ext_toml::value::Map;
use ext_toml::Value;
use wasmtime::component::Resource;

use super::interface::PluginState;
#[async_trait]
impl ConvertWithState<Value> for TomlValue {
    async fn convert_with_state(self, state: &mut PluginState) -> Value {
        match self {
            TomlValue::String(string) => Value::String(string),
            TomlValue::Integer(int) => Value::Integer(int),
            TomlValue::Float(float) => Value::Float(float),
            TomlValue::Boolean(b) => Value::Boolean(b),
            TomlValue::Datetime(datetime) => Value::Datetime(datetime.convert()),
            TomlValue::Array(array) => {
                let mut new_array = Vec::with_capacity(array.len());
                for item in array.into_iter() {
                    new_array.push(state.get_toml(item).convert_with_state(state).await)
                }
                Value::Array(new_array)
            }
            TomlValue::Table(t) => {
                let mut table = Map::new();
                for (key, value) in t {
                    let converted = state.get_toml(value).convert_with_state(state).await;
                    table.insert(key, converted);
                }
                Value::Table(table)
            }
        }
    }
}

#[async_trait]
impl ConvertWithState<TomlValue> for Value {
    async fn convert_with_state(self, state: &mut PluginState) -> TomlValue {
        match self {
            Value::String(string) => TomlValue::String(string),
            Value::Integer(int) => TomlValue::Integer(int),
            Value::Float(float) => TomlValue::Float(float),
            Value::Boolean(b) => TomlValue::Boolean(b),
            Value::Datetime(d) => TomlValue::Datetime(d.convert()),
            Value::Array(array) => {
                let mut new_arr = Vec::with_capacity(array.len());
                for item in array.into_iter() {
                    new_arr.push(item.convert_with_state(state).await);
                }
                TomlValue::Array(new_arr)
            }
            Value::Table(list) => {
                let mut table = Vec::with_capacity(list.len());
                for (key, item) in list.into_iter() {
                    table.push((key, item.convert_with_state(state).await));
                }
                TomlValue::Table(table)
            }
        }
    }
}

#[async_trait]
impl ConvertWithState<Resource<Toml>> for Value {
    async fn convert_with_state(self, state: &mut PluginState) -> Resource<Toml> {
        let toml_value: TomlValue = self.convert_with_state(state).await;
        toml_value.convert_with_state(state).await
    }
}

#[async_trait]
impl ConvertWithState<Resource<Toml>> for TomlValue {
    async fn convert_with_state(self, state: &mut PluginState) -> Resource<Toml> {
        // This impl causes the set function add whole new toml's, check if it's
        // already in state somehow?
        state.new(self).await.unwrap()
    }
}
