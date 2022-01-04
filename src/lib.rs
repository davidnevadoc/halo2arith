use halo2::plonk::Error;
use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Region},
};
use std::marker::PhantomData;
use utils::decompose;

pub mod main_gate;
pub mod utils;

pub trait Assigned<F: FieldExt> {
    fn value(&self) -> Option<F>;
    fn constrain_equal(&self, region: &mut Region<'_, F>, new_cell: Cell) -> Result<(), Error> {
        region.constrain_equal(self.cell(), new_cell)
    }
    fn cell(&self) -> Cell;
    fn decompose(&self, number_of_limbs: usize, bit_len: usize) -> Option<Vec<F>> {
        self.value().map(|e| decompose(e, number_of_limbs, bit_len))
    }
}

#[derive(Debug, Clone)]
pub struct AssignedCondition<F: FieldExt> {
    bool_value: Option<bool>,
    cell: Cell,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> From<AssignedValue<F>> for AssignedCondition<F> {
    fn from(assigned: AssignedValue<F>) -> Self {
        AssignedCondition::new(assigned.cell, assigned.value)
    }
}

impl<F: FieldExt> AssignedCondition<F> {
    fn new(cell: Cell, value: Option<F>) -> Self {
        let bool_value = value.map(|value| if value == F::zero() { false } else { true });
        AssignedCondition {
            bool_value,
            cell,
            _marker: PhantomData,
        }
    }
}

impl<F: FieldExt> Assigned<F> for AssignedCondition<F> {
    fn value(&self) -> Option<F> {
        self.bool_value
            .map(|value| if value { F::one() } else { F::zero() })
    }
    fn cell(&self) -> Cell {
        self.cell
    }
}

type AssignedBit<F> = AssignedCondition<F>;

#[derive(Debug, Clone)]
pub struct AssignedValue<F: FieldExt> {
    pub value: Option<F>,
    cell: Cell,
}

impl<F: FieldExt> From<AssignedCondition<F>> for AssignedValue<F> {
    fn from(cond: AssignedCondition<F>) -> Self {
        AssignedValue {
            value: (&cond).value(),
            cell: cond.cell,
        }
    }
}

impl<F: FieldExt> Assigned<F> for AssignedValue<F> {
    fn value(&self) -> Option<F> {
        self.value
    }
    fn cell(&self) -> Cell {
        self.cell
    }
}

impl<F: FieldExt> AssignedValue<F> {
    fn new(cell: Cell, value: Option<F>) -> Self {
        AssignedValue { value, cell }
    }
}

#[derive(Debug, Clone)]
pub struct UnassignedValue<F: FieldExt>(Option<F>);

impl<F: FieldExt> From<Option<F>> for UnassignedValue<F> {
    fn from(value: Option<F>) -> Self {
        UnassignedValue(value)
    }
}

impl<F: FieldExt> UnassignedValue<F> {
    // pub fn value(&self) -> Result<F, Error> {
    //     Ok(self.0.clone().ok_or(Error::Synthesis)?)
    // }

    pub fn value(&self) -> Option<F> {
        self.0.clone()
    }

    pub fn decompose(&self, number_of_limbs: usize, bit_len: usize) -> Option<Vec<F>> {
        self.0.map(|e| decompose(e, number_of_limbs, bit_len))
    }

    pub fn assign(&self, cell: Cell) -> AssignedValue<F> {
        AssignedValue::new(cell, self.0)
    }
}
