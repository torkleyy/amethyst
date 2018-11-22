use std::{borrow::Cow, ops::Deref};

use fnv::FnvHashMap as HashMap;
use shrev::ReaderId;
use specs::{
    shred::RunningTime,
    storage::{ComponentEvent, MaskedStorage},
    world::LazyBuilder,
    Component, DenseVecStorage, Entities, Entity, EntityBuilder, Resources, Storage, System,
    WriteStorage,
};

use util::{Cache, CachedStorage};

pub trait FindNamed {
    fn find<S>(&self, s: S) -> Option<Entity>
    where
        S: AsRef<Cow<'static, str>>;
}

impl<'e, D> FindNamed for Storage<'e, Named, D>
where
    D: Deref<Target = MaskedStorage<Named>>,
{
    fn find<S>(&self, s: S) -> Option<Entity>
    where
        S: AsRef<Cow<'static, str>>,
    {
        let entities = self.fetched_entities();

        self.unprotected_storage().cache.map.get(s.as_ref()).map(|i| entities.get(i))
    }
}

/// A component that gives a name to an [`Entity`].
///
/// There are two ways you can get a name for an entity:
///
/// * Hard-coding the entity name in code, in which case the name would be a [`&'static str`][str].
/// * Dynamically generating the string or loading it from a data file, in which case the name
///   would be a `String`.
///
/// To support both of these cases smoothly, `Named` stores the name as [`Cow<'static, str>`].
/// You can pass either a [`&'static str`][str] or a [`String`] to [`Named::new`], and your code
/// can generally treat the `name` field as a [`&str`][str] without needing to know whether the
/// name is actually an owned or borrowed string.
///
/// [`Entity`]: https://docs.rs/specs/*/specs/struct.Entity.html
/// [`Cow<'static, str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [str]: https://doc.rust-lang.org/std/primitive.str.html
/// [`Named::new`]: #method.new
///
/// # Examples
///
/// Creating a name from string constant:
///
/// ```
/// # extern crate amethyst;
/// use amethyst::core::{Named, WithNamed};
/// use amethyst::ecs::prelude::*;
///
/// let mut world = World::new();
/// world.register::<Named>();
///
/// world
///     .create_entity()
///     .named("Super Cool Entity")
///     .build();
/// ```
///
/// Creating a name from a dynamically generated string:
///
/// ```
/// # extern crate amethyst;
/// use amethyst::core::{Named, WithNamed};
/// use amethyst::ecs::prelude::*;
///
/// let mut world = World::new();
/// world.register::<Named>();
///
/// for entity_num in 0..10 {
///     world
///         .create_entity()
///         .named(format!("Entity Number {}", entity_num))
///         .build();
/// }
/// ```
///
/// Accessing a named entity in a system:
///
/// ```
/// # extern crate amethyst;
/// use amethyst::core::Named;
/// use amethyst::ecs::prelude::*;
///
/// pub struct NameSystem;
///
/// impl<'s> System<'s> for NameSystem {
///     type SystemData = (
///         Entities<'s>,
///         ReadStorage<'s, Named>,
///     );
///
///     fn run(&mut self, (entities, names): Self::SystemData) {
///         for (entity, name) in (&*entities, &names).join() {
///             println!("Entity {:?} is named {}", entity, name.name);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Named {
    /// The name of the entity this component is attached to.
    pub name: Cow<'static, str>,
}

impl Named {
    /// Constructs a new `Named` from a string.
    ///
    /// # Examples
    ///
    /// From a string constant:
    ///
    /// ```
    /// # extern crate amethyst;
    /// use amethyst::core::Named;
    ///
    /// let name_component = Named::new("Super Cool Entity");
    /// ```
    ///
    /// From a dynamic string:
    ///
    /// ```
    /// # extern crate amethyst;
    /// use amethyst::core::Named;
    ///
    /// let entity_num = 7;
    /// let name_component = Named::new(format!("Entity Number {}", entity_num));
    /// ```
    pub fn new<S>(name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Named { name: name.into() }
    }
}

impl Component for Named {
    type Storage = CachedStorage<NameCache, DenseVecStorage<Self>, Self>;
}

/// An easy way to name an `Entity` and give it a `Named` `Component`.
pub trait WithNamed
where
    Self: Sized,
{
    /// Adds a name to the entity being built.
    fn named<S>(self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>;
}

impl<'a> WithNamed for EntityBuilder<'a> {
    fn named<S>(self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.world
            .system_data::<(WriteStorage<'a, Named>,)>()
            .0
            .insert(self.entity, Named::new(name))
            .expect("Unreachable: Entities should always be valid when just created");
        self
    }
}

impl<'a> WithNamed for LazyBuilder<'a> {
    fn named<S>(self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.lazy.insert::<Named>(self.entity, Named::new(name));
        self
    }
}

pub struct NameCache {
    map: HashMap<Cow<'static, str>, u32>,
}

impl Cache<Named> for NameCache {
    fn on_get(&self, _: u32, _: &Named) {}

    fn on_update(&mut self, id: u32, val: &Named) {
        self.map.insert(val.name.clone(), id);
    }

    fn on_remove(&mut self, id: u32, val: Named) -> Named {
        self.map.remove(&val.name);

        val
    }
}
