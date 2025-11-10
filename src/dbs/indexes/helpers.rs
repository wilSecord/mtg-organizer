macro_rules! count {
    ( $h:ident $($t:ident)* ) => {
        1 + $crate::dbs::indexes::helpers::count!($($t)*)
    };
    () => { 0 }
}

macro_rules! first {
    ( $h:ident $($t:ident)* ) => {
        $h
    };
    () => {};
}

macro_rules! if_empty_else {
    ($true:tt $false:tt) => {
        $true
    };
    ($i:ident $true:tt $false:tt) => {
        $false
    }

}

macro_rules! make_index_types {
    (
        key $keyname:ident {
            $( 
                $(#[$serde_kind:ident])?
                $fieldname:ident: $fieldtype:ty $(,)? 
            )*
        }
    ) => {
        #[allow(non_snake_case)]
        pub mod $keyname {
            use minimal_storage::serialize_min::{SerializeMinimal, DeserializeFromMinimal};
            use minimal_storage::serialize_fast::MinimalSerdeFast;

            const DIMENSIONS: usize = $crate::dbs::indexes::helpers::count!( $( $fieldname )* );

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct Key {
                $( pub $fieldname : $fieldtype, )*
            }
            impl std::cmp::PartialOrd for Key {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }
            impl std::cmp::Ord for Key {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    $(
                        let $fieldname = self.$fieldname.cmp(&other.$fieldname);
                        if $fieldname != std::cmp::Ordering::Equal {
                            return $fieldname;
                        }
                    )*
                    return std::cmp::Ordering::Equal;
                }
            }
            impl tree::tree_traits::MinValue for Key {
                const MIN: Self = Self {
                    $( $fieldname: tree::tree_traits::MinValue::MIN, )*
                };
            }
            impl tree::tree_traits::MultidimensionalKey<DIMENSIONS> for Key {
                type Parent = Query;
                type DeltaFromParent = Self;
                type DeltaFromSelfAsChild = Self;

                fn is_contained_in(&self, parent: &Self::Parent) -> bool {
                    true
                    $( && self.$fieldname.is_contained_in(&parent.$fieldname) )*
                }
                fn delta_from_parent(&self, _parent: &Self::Parent) -> Self::DeltaFromParent {
                    self.to_owned()
                }

                fn apply_delta_from_parent(delta: &Self::DeltaFromParent, _parent: &Self::Parent) -> Self {
                    delta.to_owned()
                }
                fn smallest_key_in(parent: &Self::Parent) -> Self {
                    Self {
                        $( $fieldname: *parent.$fieldname.start(), )*
                    }
                }
                fn largest_key_in(parent: &Self::Parent) -> Self {
                    Self {
                        $( $fieldname: *parent.$fieldname.end(), )*
                    }
                }
                fn delta_from_self(
                    finl: &Self::DeltaFromParent,
                    _initil: &Self::DeltaFromParent,
                ) -> Self::DeltaFromSelfAsChild {
                    finl.to_owned()
                }

                fn apply_delta_from_self(
                    delta: &Self::DeltaFromSelfAsChild,
                    _initial: &Self::DeltaFromParent,
                ) -> Self::DeltaFromParent {
                    delta.to_owned()
                }
            }
            impl SerializeMinimal for Key {
                type ExternalData<'s> = ();

                fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
                    &'a self,
                    write_to: &mut W,
                    _: Self::ExternalData<'s>,
                ) -> std::io::Result<()> {
                    $( self.$fieldname.minimally_serialize(write_to, ())?; )*
                    Ok(())
                }
            }
            impl DeserializeFromMinimal for Key {
                type ExternalData<'d> = ();

                fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
                    from: &'a mut R,
                    _: Self::ExternalData<'d>,
                ) -> Result<Self, std::io::Error> {
                    $( let $fieldname = <$fieldtype>::deserialize_minimal(from, ())?; )*

                    Ok(Self {
                        $( $fieldname, )*
                    })
                }
            }
            impl MinimalSerdeFast for Key {
                fn fast_minimally_serialize<'a, 's: 'a, W: std::io::Write>(
                    &'a self,
                    write_to: &mut W,
                    _external_data: <Self as SerializeMinimal>::ExternalData<'s>,
                ) -> std::io::Result<()> {
                    $(
                        $crate::dbs::indexes::helpers::if_empty_else!($($serde_kind)? {
                            self.$fieldname.minimally_serialize(write_to, ())?
                        } {
                            self.$fieldname.fast_minimally_serialize(write_to, ())?
                        });
                    )*
                    
                    Ok(())
                }

                fn fast_deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
                    from: &'a mut R,
                    _external_data: <Self as DeserializeFromMinimal>::ExternalData<'d>,
                ) -> Result<Self, std::io::Error> {
                    $(
                        let $fieldname = $crate::dbs::indexes::helpers::if_empty_else!($($serde_kind)? {
                            <$fieldtype>::deserialize_minimal(from, ())?
                        } {
                            <$fieldtype>::fast_deserialize_minimal(from, ())?
                        });
                    )*

                    Ok(Self {
                        $( $fieldname, )*
                    })
                }

                fn fast_seek_after<R: std::io::Read>(from: &mut R) -> std::io::Result<()> {
                    
                    Self::fast_deserialize_minimal(from, ()).map(|_| ())
                }
            }
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct Query {
                $( pub $fieldname : std::ops::RangeInclusive<$fieldtype>, )*
            }

            #[derive(Debug, Clone, Copy)]
            #[repr(usize)]
            pub enum Dim {
                $( #[allow(non_camel_case_types)] $fieldname, )*
            }

            impl SerializeMinimal for Query {
                type ExternalData<'s> = ();

                fn minimally_serialize<'a, 's: 'a, W: std::io::Write>(
                    &'a self,
                    write_to: &mut W,
                    _: Self::ExternalData<'s>,
                ) -> std::io::Result<()> {
                    $( self.$fieldname.minimally_serialize(write_to, ())?; )*
                    Ok(())
                }
            }
            impl DeserializeFromMinimal for Query {
                type ExternalData<'d> = ();

                fn deserialize_minimal<'a, 'd: 'a, R: std::io::Read>(
                    from: &'a mut R,
                    _: Self::ExternalData<'d>,
                ) -> Result<Self, std::io::Error> {
                    $( let $fieldname = <std::ops::RangeInclusive<$fieldtype>>::deserialize_minimal(from, ())?; )*

                    Ok(Self {
                        $( $fieldname, )*
                    })
                }
            }

            impl tree::tree_traits::Dimension<DIMENSIONS> for Dim {
                fn next_axis(&self) -> Self {
                    Self::from_index(*self as usize + 1)
                }
                fn from_index(i: usize) -> Self {
                    let safe_idx = i % DIMENSIONS;
                    //Safe because the index is always in 0..DIMENSIONS
                    //and because [1] defines enum discriminants as starting at 0
                    //and progressing up by 1 each case.
                    //[1]: https://doc.rust-lang.org/reference/items/enumerations.html#r-items.enum.discriminant.implicit
                    unsafe { std::mem::transmute(safe_idx) }
                }
                fn arbitrary_first() -> Self {
                    use Dim::*;
                    $crate::dbs::indexes::helpers::first!( $( $fieldname )* )
                }
            }

            impl tree::tree_traits::MultidimensionalParent<DIMENSIONS> for Query {
                type DimensionEnum = Dim;

                const UNIVERSE: Self = Self {
                    $( $fieldname: <std::ops::RangeInclusive<$fieldtype>>::UNIVERSE, )*
                };

                fn contains(&self, child: &Self) -> bool {
                    true
                    $( && tree::tree_traits::MultidimensionalParent::contains(&self.$fieldname, &child.$fieldname) )*
                }
                fn overlaps(&self, child: &Self) -> bool {
                    false
                    $( || self.$fieldname.overlaps(&child.$fieldname) )*
                }

                fn split_evenly_on_dimension(&self, dimension: &Self::DimensionEnum) -> (Self, Self) {
                    let mut left = self.to_owned();
                    let mut right = self.to_owned();

                    match dimension {
                        $( Dim::$fieldname => {
                            let (fl, fr) = self.$fieldname.split_evenly_on_dimension(&());
                            left.$fieldname = fl;
                            right.$fieldname = fr;
                        } )*
                    }

                    (left, right)
                }
            }
        }

    };
}
pub(crate) use count;
pub(crate) use first;
pub(crate) use if_empty_else;
pub(crate) use make_index_types;