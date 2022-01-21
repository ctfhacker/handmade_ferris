//! Implementation of <https://github.com/eqv/rand_romu>

/// Implementation of `RandRomu`
#[derive(Debug)]
pub struct Rng {
    /// X state
    xstate: u64,

    /// Y state
    ystate: u64,
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng {
    /// Create a new [`Rng`] seeded by a number from a `Lehmer64` rng
    pub fn new() -> Rng {
        // Generate the random state from Lehmer64
        let mut lehmer64 = Lehmer64::new();
        let mut res = Rng {
            xstate: lehmer64.next(),
            ystate: lehmer64.next(),
        };

        // Cycle through to create some chaos
        for _ in 0..100 {
            let _ = res.next();
        }

        res
    }

    /// Get the next number from the rng
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        let xp = self.xstate;
        self.xstate = 15_241_094_284_759_029_579u64.wrapping_mul(self.ystate);
        self.ystate = self.ystate.wrapping_sub(xp);
        self.ystate = self.ystate.rotate_left(27);
        xp
    }
}

/// Rng seeded with rdtsc that is generated using Lehmer64
pub struct Lehmer64 {
    /// Current state
    value: u128,
}

impl Lehmer64 {
    /// Create a new [`Lehmer64`] rng seeded by `rdtsc`
    pub fn new() -> Lehmer64 {
        let mut res = Lehmer64 {
            value: u128::from(unsafe { core::arch::x86_64::_rdtsc() })
        };

        // Cycle through to create some chaos
        for _ in 0..100 {
            let _ = res.next();
        }

        res
    }

    /// Get the next number from the rng
    pub fn next(&mut self) -> u64 {
        self.value = self.value.wrapping_mul(0xda94_2042_e4dd_58b5);
        (self.value >> 64) as u64
    }
}

/// Add supplemental functions to an enum to allow for random element selection
#[macro_export]
macro_rules! random_enum {
    ( /* Match case */
        $(#[$attr:meta])* 
        pub enum $name:ident { 
            $( 
                $(#[$inner:ident $($args:tt)*])*
                $field_vis:vis $variant:ident,
            )* $(,)? 
        }
    ) => {
        $(#[$attr])*
        pub enum $name { 
            $(
                $(#[$inner $($args)*])*
                $field_vis $variant,
            )* 
        }

        impl $name {
            pub const fn elements() -> &'static [$name] {
                &[$( $name::$variant,)*]
            }

            pub const fn len() -> usize {
                $name::elements().len()
            }

            pub fn rand(rng: &mut Rng) -> $name {
                let index = rng.next() % $name::len() as u64;
                $name::elements()[index as usize]
            }
        }
    };

    (   
        $(#[$attr:meta])* 
        pub enum $name:ident { 
            $( 
                $(#[$inner:ident $($args:tt)*])*
                $field_vis:vis $variant:ident = $val:expr,
            )* $(,)? 
        }
    ) => {
        $(#[$attr])*
        pub enum $name { 
            $(
                $(#[$inner $($args)*])*
                $field_vis $variant = $val,
            )* 
        }

        impl $name {
            pub const fn elements() -> &'static [$name] {
                &[$( $name::$variant,)*]
            }

            pub const fn len() -> usize {
                $name::elements().len()
            }

            /// Get a random variant of `$name`
            pub fn rand(rng: &mut Rng) -> $name {
                let index = rng.next() % $name::len() as u64;
                $name::elements()[index as usize]
            }
        }
    }
}
