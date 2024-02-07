use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use crate::globals::Globals;

#[derive(Clone)]
pub struct Unit {
    pub id: String,
}
impl Unit {
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn none() -> Unit {
        Unit {
            id: "".to_string()
        }
    }

    pub fn from_id(id: &str) -> Unit {
        Unit {
            id: id.to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum UnitProperty { ANGLE, LENGTH, TEMP, MassWeight, DURATION, VOLUME, CURRENT, VOLTAGE, RESISTANCE, UNDEFINED }

#[derive(PartialEq)]
pub enum UnitTag { DateTime, ShortDateTime, LongDateTime }
pub struct UnitDef {
    pub to_si_factor: f64,
    pub id: String,
    pub si_id: &'static str,
    pub property: UnitProperty,
    to_si_fn: fn(&UnitDef, f64) -> f64,
    from_si_fn: fn(&UnitDef, f64) -> f64,
    pub tags: &'static [UnitTag],
}

impl UnitDef {
    pub fn new(id: &str, si_id: &'static str, to_si_factor: f64, property: UnitProperty, tags: &'static [UnitTag]) -> UnitDef {
        UnitDef {
            to_si_factor,
            id: id.to_string(),
            si_id,
            property,
            to_si_fn: default_to_si,
            from_si_fn: default_from_si,
            tags,
        }
    }

    pub fn convert_to_si(&self, value: f64) -> f64 {
        (self.to_si_fn)(self, value)
    }

    pub fn convert_from_si(&self, value: f64) -> f64 {
        (self.from_si_fn)(self, value)
    }
}

#[derive(Clone)]
pub struct UnitsView {
    pub units: HashSet<String>,
}

impl UnitsView {
    pub fn new() -> Self {
        UnitsView {
            units: HashSet::new(),
        }
    }

    pub fn get_def<'a>(&'a self, id: &str, globals: &'a Globals) -> Option<&UnitDef> {
        if self.units.contains(id) {
            return Some(&globals.unit_defs[id])
        }
        None
    }

    pub fn add_class(&mut self, property: &UnitProperty, unit_defs: &HashMap<String, UnitDef>) {
        self.units.extend(unit_defs
            .values()
            .filter(|unit| &unit.property == property )
            .map(|unit| unit.id.to_string())
        );
    }

    pub fn add_tagged(&mut self, tag: &UnitTag, globals: &Globals) {
        self.units.extend(globals.unit_defs
            .values()
            .filter(|unit| unit.tags.contains(&tag) )
            .map(|unit| unit.id.to_string())
        );
    }

    pub fn remove_class(&mut self, property: &UnitProperty, globals: &Globals) {
        self.units.retain(|unit| &globals.unit_defs[unit].property != property);
    }

    pub fn remove_tagged(&mut self, tag: UnitTag, unit_defs: &HashMap<String, UnitDef>) {
        self.units.retain(|unit| unit_defs[unit].tags.contains(&tag) == false);
    }

    pub fn add_default_classes(&mut self, unit_defs: &HashMap<String, UnitDef>) {
        self.add_class(&UnitProperty::UNDEFINED, unit_defs); //needed to include the empty unit.
        self.add_class(&UnitProperty::ANGLE, unit_defs);
        self.add_class(&UnitProperty::LENGTH, unit_defs);
        self.add_class(&UnitProperty::TEMP, unit_defs);
        self.add_class(&UnitProperty::MassWeight, unit_defs);
        self.add_class(&UnitProperty::DURATION, unit_defs);
        self.add_class(&UnitProperty::VOLUME, unit_defs);
        //electricity not set by default.
    }
}

pub fn default_to_si(def: &UnitDef, from: f64) -> f64 {
    from * def.to_si_factor
}
pub fn default_from_si(def: &UnitDef, from: f64) -> f64 {
    from / def.to_si_factor
}

#[inline]
fn insert_def(defs: &mut HashMap<String, UnitDef>, id: &str, si_id: &'static str, to_si_factor: f64, property: UnitProperty, tags: &'static [UnitTag]) {
    defs.insert(id.to_string(), UnitDef::new(id, si_id, to_si_factor, property, tags));
}

pub fn create_unit_defs() -> HashMap<String, UnitDef> {
    let mut defs: HashMap<String, UnitDef> = HashMap::new();

    insert_def(&mut defs, "", "", 1.0, UnitProperty::UNDEFINED, &[]);
    insert_def(&mut defs, "rad", "rad", 1.0, UnitProperty::ANGLE, &[]);
    insert_def(&mut defs, "deg", "rad", PI / 180.0, UnitProperty::ANGLE, &[]);

    insert_def(&mut defs, "m", "m", 1.0, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "cm", "m", 0.01, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "km", "m", 1000.0, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "mm", "m", 0.001, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "in", "m", 0.0254, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "ft", "m", 0.3048, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "thou", "m", 0.0254 / 1000.0, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "yd", "m", 0.9144, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "mi", "m", 1609.344, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "micron", "m", 0.000001, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "um", "m", 0.000001, UnitProperty::LENGTH, &[]);
    insert_def(&mut defs, "ly", "m", 9460730472580800.0, UnitProperty::LENGTH, &[]);

    insert_def(&mut defs, "C", "K", 0.000001, UnitProperty::TEMP, &[]);
    insert_def(&mut defs, "K", "K", 1.0, UnitProperty::TEMP, &[]);
    insert_def(&mut defs, "F", "K", 0.000001, UnitProperty::TEMP, &[]);

    insert_def(&mut defs, "L", "L", 1.0, UnitProperty::VOLUME, &[]);
    insert_def(&mut defs, "mL", "L", 0.001, UnitProperty::VOLUME, &[]);
        //ml, with lower case l is non standard
    insert_def(&mut defs, "ml", "L", 0.001, UnitProperty::VOLUME, &[]);
    insert_def(&mut defs, "gal", "L", 3.785411784, UnitProperty::VOLUME, &[]);
    insert_def(&mut defs, "pt", "L", 0.473176473, UnitProperty::VOLUME, &[]);

    insert_def(&mut defs, "kg", "kg", 1.0, UnitProperty::MassWeight, &[]);
    insert_def(&mut defs, "N", "kg", 1.0/9.80665, UnitProperty::MassWeight, &[]);
    insert_def(&mut defs, "g", "kg", 0.001, UnitProperty::MassWeight, &[]);
    insert_def(&mut defs, "mg", "kg", 0.000001, UnitProperty::MassWeight, &[]);
    insert_def(&mut defs, "t", "kg", 1000.0, UnitProperty::MassWeight, &[]);
    insert_def(&mut defs, "lb", "kg", 0.45359, UnitProperty::MassWeight, &[]);
    insert_def(&mut defs, "lbs", "kg", 0.45359, UnitProperty::MassWeight, &[]);
    insert_def(&mut defs, "oz", "kg", 1.0/ 35.2739619496, UnitProperty::MassWeight, &[]);

    insert_def(&mut defs, "seconds","s", 1.0, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "minutes", "s", 60.0, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "hours", "s", 3600.0, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "days", "s", 86400.0, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "weeks", "s", (60 * 60 * 24 * 7) as f64, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "months", "s", 2629746.0, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "years", "s", 31556952.0, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "milliseconds", "s", 1.0/1000.0, UnitProperty::DURATION, &[UnitTag::LongDateTime, UnitTag::DateTime]);

    insert_def(&mut defs, "s", "s", 1.0, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "min", "s", 60.0, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "h", "s", 3600.0, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "d", "s", 86400.0, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "w", "s", (60 * 60 * 24 * 7) as f64, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "mon", "s", 2629746.0, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "y", "s", 31556952.0, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);
    insert_def(&mut defs, "ms", "s", 1.0/1000.0, UnitProperty::DURATION, &[UnitTag::ShortDateTime, UnitTag::DateTime]);

    insert_def(&mut defs, "A", "A", 1.0, UnitProperty::CURRENT, &[]);
    insert_def(&mut defs, "mA", "A", 0.001, UnitProperty::CURRENT, &[]);

    insert_def(&mut defs, "R", "R", 1.0, UnitProperty::RESISTANCE, &[]);
    insert_def(&mut defs, "mR", "R", 0.001, UnitProperty::RESISTANCE, &[]);
    insert_def(&mut defs, "kR", "R", 1000.0, UnitProperty::RESISTANCE, &[]);
    insert_def(&mut defs, "MR", "R", 1000000.0, UnitProperty::RESISTANCE, &[]);

    insert_def(&mut defs, "V", "V", 1.0, UnitProperty::VOLTAGE, &[]);
    insert_def(&mut defs, "mV", "V", 0.001, UnitProperty::VOLTAGE, &[]);

    if let Some(def) = defs.get_mut("C") {
        def.to_si_fn = |_ud, d| d + 273.15;
        def.from_si_fn = |_ud, d| d - 273.15;
    }
    if let Some(def) = defs.get_mut("F") {
        def.to_si_fn = |_ud, d| (d - 32.0) * 5.0 / 9.0 + 273.15;
        def.from_si_fn = |_ud, d| (d - 273.15) * 9.0 / 5.0 + 32.0;
    }
    defs
}

#[cfg(test)]
mod tests {
    use crate::globals::Globals;
    use crate::resolver::unit::{UnitProperty, UnitsView};

    #[test]
    fn test_units() {
        let globals = Globals::new();
        let mut view = UnitsView::new();
        view.units.clear();
        view.add_class(&UnitProperty::ANGLE, &globals.unit_defs);
        assert_eq!(view.units.len(), 2);
    }

    #[test]
    fn test_clone_units() {
        let globals = Globals::new();
        let mut view = UnitsView::new();
        view.units.clear();
        view.add_class(&UnitProperty::ANGLE, &globals.unit_defs);
        view.add_class(&UnitProperty::TEMP, &globals.unit_defs);
        assert_eq!(view.units.len(), 5);

        view.remove_class(&UnitProperty::ANGLE, &globals);
        assert_eq!(view.units.len(), 3);
    }
}
