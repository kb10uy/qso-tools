use mlua::prelude::*;
use time::{
    Date, OffsetDateTime, Time, UtcOffset,
    format_description::{BorrowedFormatItem, parse_borrowed, well_known::Rfc3339},
    macros::format_description,
};

use crate::schope::library::SchopeModule;

const DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[year]-[month]-[day]");
const TIME_FORMAT: &[BorrowedFormatItem] = format_description!("[hour]:[minute]:[second]");
const OFFSET_FORMAT: &[BorrowedFormatItem] = format_description!("[offset_hour]:[offset_minute]");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SchopeDateTime(OffsetDateTime);

impl SchopeDateTime {
    fn to_utc(self) -> SchopeDateTime {
        SchopeDateTime(self.0.to_offset(UtcOffset::UTC))
    }

    fn to_offset(self, offset_str: &str) -> LuaResult<SchopeDateTime> {
        Ok(SchopeDateTime(self.0.to_offset(parse_offset(offset_str)?)))
    }

    fn format(self, format: &str) -> LuaResult<String> {
        let format = parse_borrowed::<1>(format).map_err(LuaError::external)?;
        self.0.format(&format).map_err(LuaError::external)
    }

    fn date_str(self) -> LuaResult<String> {
        self.0.format(DATE_FORMAT).map_err(LuaError::external)
    }

    fn time_str(self) -> LuaResult<String> {
        self.0.format(TIME_FORMAT).map_err(LuaError::external)
    }
}

impl LuaUserData for SchopeDateTime {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("to_utc", |_, this, ()| Ok(this.to_utc()));
        methods.add_method("to_offset", |_, this, o: String| this.to_offset(&o));
        methods.add_method("format", |_, this, f: String| this.format(&f));

        methods.add_meta_method(
            LuaMetaMethod::Eq,
            |_, lhs, rhs: LuaUserDataRef<SchopeDateTime>| Ok(*lhs == *rhs),
        );
        methods.add_meta_method(
            LuaMetaMethod::Lt,
            |_, lhs, rhs: LuaUserDataRef<SchopeDateTime>| Ok(*lhs < *rhs),
        );
        methods.add_meta_method(
            LuaMetaMethod::Le,
            |_, lhs, rhs: LuaUserDataRef<SchopeDateTime>| Ok(*lhs <= *rhs),
        );
    }

    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("year", |_, this| Ok(this.0.year()));
        fields.add_field_method_get("month", |_, this| Ok(this.0.month() as u8));
        fields.add_field_method_get("day", |_, this| Ok(this.0.day()));
        fields.add_field_method_get("hour", |_, this| Ok(this.0.hour()));
        fields.add_field_method_get("minute", |_, this| Ok(this.0.minute()));
        fields.add_field_method_get("second", |_, this| Ok(this.0.second()));

        fields.add_field_method_get("date_str", |_, this| this.date_str());
        fields.add_field_method_get("time_str", |_, this| this.time_str());
    }
}

impl From<OffsetDateTime> for SchopeDateTime {
    fn from(value: OffsetDateTime) -> Self {
        SchopeDateTime(value)
    }
}

impl From<SchopeDateTime> for OffsetDateTime {
    fn from(value: SchopeDateTime) -> Self {
        value.0
    }
}

fn parse_date(date_str: &str) -> LuaResult<Date> {
    Date::parse(date_str, DATE_FORMAT).map_err(LuaError::external)
}

fn parse_time(time_str: &str) -> LuaResult<Time> {
    Time::parse(time_str, TIME_FORMAT).map_err(LuaError::external)
}

fn parse_offset(offset_str: &str) -> LuaResult<UtcOffset> {
    UtcOffset::parse(offset_str, OFFSET_FORMAT).map_err(LuaError::external)
}

pub struct DateTimeModule;

impl DateTimeModule {
    fn now_utc() -> SchopeDateTime {
        SchopeDateTime(OffsetDateTime::now_utc())
    }

    fn now_local() -> LuaResult<SchopeDateTime> {
        OffsetDateTime::now_local()
            .map(SchopeDateTime)
            .map_err(LuaError::external)
    }

    fn from_rfc3339(dt_str: &str) -> LuaResult<SchopeDateTime> {
        OffsetDateTime::parse(dt_str, &Rfc3339)
            .map(SchopeDateTime)
            .map_err(LuaError::external)
    }

    fn from_parts_utc(date_str: &str, time_str: &str) -> LuaResult<SchopeDateTime> {
        Ok(SchopeDateTime(OffsetDateTime::new_utc(
            parse_date(date_str)?,
            parse_time(time_str)?,
        )))
    }

    fn from_parts_offset(
        date_str: &str,
        time_str: &str,
        offset_str: &str,
    ) -> LuaResult<SchopeDateTime> {
        Ok(SchopeDateTime(OffsetDateTime::new_in_offset(
            parse_date(date_str)?,
            parse_time(time_str)?,
            parse_offset(offset_str)?,
        )))
    }
}

impl SchopeModule for DateTimeModule {
    fn create_module_table(lua: &Lua, _: LuaMultiValue) -> LuaResult<LuaTable> {
        let t = lua.create_table()?;

        t.set("now_utc", lua.create_function(|_, ()| Ok(Self::now_utc()))?)?;
        t.set("now_local", lua.create_function(|_, ()| Self::now_local())?)?;
        t.set(
            "from_rfc3339",
            lua.create_function(|_, s: String| Self::from_rfc3339(&s))?,
        )?;
        t.set(
            "from_parts_utc",
            lua.create_function(|_, (d, t): (String, String)| Self::from_parts_utc(&d, &t))?,
        )?;
        t.set(
            "from_parts_offset",
            lua.create_function(|_, (d, t, o): (String, String, String)| {
                Self::from_parts_offset(&d, &t, &o)
            })?,
        )?;

        Ok(t)
    }
}
