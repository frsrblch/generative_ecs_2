use crate::lifetimes::*;
use code_gen::Visibility::Pub;
use code_gen::*;
use std::fmt::*;
use std::marker::PhantomData;
use std::str::FromStr;

// TODO replace generic methods with type strings, it doesn't work with type aliases as found in physics

#[derive(Debug, Clone)]
pub struct Component {
    pub field_name: SnakeCase,
    pub source: Source,
    pub density: Density,
    pub comp_type: Type,
}

impl Component {
    pub fn get_component_type(&self) -> Type {
        Type::new(&format!("Component<Self,{}>", self.get_type()))
    }

    pub fn get_explicit_component_type(&self, arena: &ArenaName) -> Type {
        Type::new(&format!("Component<{},{}>", arena, self.get_type()))
    }

    pub fn get_row_field(&self) -> Option<Field> {
        self.get_row_type().map(|t| Field {
            visibility: Pub,
            name: self.field_name.clone(),
            field_type: t,
        })
    }

    fn get_row_type(&self) -> Option<Type> {
        match self.source {
            Source::ByDefault => None,
            Source::ByValue => Some(self.get_type()),
        }
    }

    fn get_type(&self) -> Type {
        match self.density {
            Density::Dense => self.comp_type.clone(),
            Density::Sparse => Type::new(&format!("Option<{}>", &self.comp_type)),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Source {
    ByValue,
    ByDefault,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Density {
    Dense,
    Sparse,
}

#[derive(Debug)]
pub struct ArenaCore {
    pub name: ArenaName,
    pub components: Vec<Component>,
    pub refs: Vec<(SnakeCase, ArenaName)>,
    pub optional_refs: Vec<(SnakeCase, ArenaName)>,
}

impl ArenaCore {
    pub fn new(name: &str) -> Self {
        Self {
            name: ArenaName::new(name),
            components: vec![],
            refs: vec![],
            optional_refs: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Arena<L: Lifetime> {
    pub arena: ArenaCore,
    marker: PhantomData<L>,
}

impl<L: Lifetime> Arena<L> {
    pub fn new(name: &str) -> Self {
        Self {
            arena: ArenaCore::new(name),
            marker: PhantomData,
        }
    }

    pub fn add_required_component(&mut self, type_name: &str) {
        let field = CamelCase::from_str(type_name)
            .map(|cc| cc.into_snake_case())
            .or_else(|_| SnakeCase::from_str(type_name))
            .expect(&format!(
                "Given type cannot be formatted as snake_case: {}",
                type_name
            ));

        self.arena.components.push(Component {
            field_name: field.clone(),
            source: Source::ByValue,
            density: Density::Dense,
            comp_type: Type::new(type_name),
        });
    }

    pub fn add_required_component_with_field(&mut self, field: &str, type_name: &str) {
        self.arena.components.push(Component {
            field_name: field
                .parse()
                .expect(&format!("Fields must be in snake_case: {}", field)),
            source: Source::ByValue,
            density: Density::Dense,
            comp_type: Type::new(type_name),
        });
    }

    pub fn add_optional_component(&mut self, type_name: &str) {
        let field = CamelCase::from_str(type_name)
            .map(|cc| cc.into_snake_case())
            .or_else(|_| SnakeCase::from_str(type_name))
            .expect(&format!(
                "Given type cannot be formatted as snake_case: {}",
                type_name
            ));

        self.arena.components.push(Component {
            field_name: field.clone(),
            source: Source::ByValue,
            density: Density::Sparse,
            comp_type: Type::new(type_name),
        });
    }

    pub fn add_optional_component_with_field(&mut self, field: &str, type_name: &str) {
        self.arena.components.push(Component {
            field_name: field
                .parse()
                .expect(&format!("Fields must be in snake_case: {}", field)),
            source: Source::ByValue,
            density: Density::Sparse,
            comp_type: Type::new(type_name),
        });
    }

    pub fn add_default_component(&mut self, type_name: &str) {
        let field = CamelCase::from_str(type_name)
            .map(|cc| cc.into_snake_case())
            .or_else(|_| SnakeCase::from_str(type_name))
            .expect(&format!(
                "Given type cannot be formatted as snake_case: {}",
                type_name
            ));

        self.arena.components.push(Component {
            field_name: field.clone(),
            source: Source::ByDefault,
            density: Density::Dense,
            comp_type: Type::new(type_name),
        });
    }

    pub fn add_default_component_with_field(&mut self, field: &str, type_name: &str) {
        self.arena.components.push(Component {
            field_name: field
                .parse()
                .expect(&format!("Fields must be in snake_case: {}", field)),
            source: Source::ByDefault,
            density: Density::Dense,
            comp_type: Type::new(type_name),
        });
    }

    pub fn add_optional_self_link(&mut self, field: &str) {
        self.arena.components.push(Component {
            field_name: field
                .parse()
                .expect(&format!("Fields must be in snake_case: {}", field)),
            source: Source::ByValue,
            density: Density::Sparse,
            comp_type: self.id_type(),
        });
    }

    pub fn add_reference(&mut self, arena: &Arena<impl Lifetime>) {
        self.arena.components.push(Component {
            field_name: arena.name().as_field_name(),
            source: Source::ByValue,
            density: Density::Dense,
            comp_type: arena.id_type(),
        });
    }

    pub fn add_reference_with_field(&mut self, field: &str, arena: &Arena<impl Lifetime>) {
        self.arena.components.push(Component {
            field_name: field
                .parse()
                .expect(&format!("Fields must be in snake_case: {}", field)),
            source: Source::ByValue,
            density: Density::Dense,
            comp_type: arena.id_type(),
        });
    }

    pub fn add_optional_reference(&mut self, arena: &Arena<impl Lifetime>) {
        self.arena.components.push(Component {
            field_name: arena.name().as_field_name(),
            source: Source::ByValue,
            density: Density::Sparse,
            comp_type: arena.id_type(),
        });
    }

    pub fn add_optional_reference_with_field(&mut self, field: &str, arena: &Arena<impl Lifetime>) {
        self.arena.components.push(Component {
            field_name: field
                .parse()
                .expect(&format!("Fields must be in snake_case: {}", field)),
            source: Source::ByValue,
            density: Density::Sparse,
            comp_type: arena.id_type(),
        });
    }

    pub fn allocator(&self) -> Type {
        L::allocator(&self.arena.name)
    }

    pub fn id_type(&self) -> Type {
        L::id_type(&self.arena.name)
    }

    pub fn valid_id_type(&self) -> Type {
        L::valid_id_type(&self.arena.name)
    }

    pub fn name(&self) -> ArenaName {
        self.arena.name.clone()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ArenaName(CamelCase);

impl ArenaName {
    pub fn new(s: &str) -> Self {
        s.parse()
            .map(Self)
            .expect(&format!("Arena names must be in CamelCase: {}", s))
    }

    pub fn as_field_name(&self) -> SnakeCase {
        self.0.into_snake_case()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn get_row_type(&self) -> Type {
        Type::new(&format!("{}Row", self))
    }

    pub fn as_type(&self) -> Type {
        self.0.clone().into()
    }
}

impl Display for ArenaName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn testing() {
        let mut body = Arena::<Permanent>::new("Body");
        body.add_required_component("Mass");
        body.add_required_component("Position");
        body.add_required_component_with_field("radius", "Length");
        body.add_default_component_with_field("name", "String");

        dbg!(body);
        //        assert!(false);
    }
}
