use crate::halo2::arithmetic::FieldExt;
use crate::halo2::circuit::Region;
use crate::halo2::plonk::{Advice, Column, ConstraintSystem, Error, Fixed};
use crate::halo2::poly::Rotation;
use crate::main_gate::{CombinationOptionCommon, MainGateInstructions, Term};
use crate::{Assigned, AssignedBit, AssignedCondition, AssignedValue, UnassignedValue};
use std::marker::PhantomData;

const WIDTH: usize = 4;

pub enum MainGateColumn {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

type CombinedValues<F> = (
    AssignedValue<F>,
    AssignedValue<F>,
    AssignedValue<F>,
    AssignedValue<F>,
);

#[derive(Clone, Debug)]
pub struct MainGateConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub c: Column<Advice>,
    pub d: Column<Advice>,

    pub sa: Column<Fixed>,
    pub sb: Column<Fixed>,
    pub sc: Column<Fixed>,
    pub sd: Column<Fixed>,
    pub sd_next: Column<Fixed>,
    pub s_mul: Column<Fixed>,
    pub s_constant: Column<Fixed>,
}

pub struct MainGate<F: FieldExt> {
    pub config: MainGateConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone, Debug)]
pub enum CombinationOption<F: FieldExt> {
    Common(CombinationOptionCommon<F>),
}

impl<F: FieldExt> From<CombinationOptionCommon<F>> for CombinationOption<F> {
    fn from(option: CombinationOptionCommon<F>) -> Self {
        CombinationOption::Common(option)
    }
}

impl<F: FieldExt> MainGateInstructions<F, WIDTH> for MainGate<F> {
    type CombinationOption = CombinationOption<F>;
    type CombinedValues = CombinedValues<F>;
    type MainGateColumn = MainGateColumn;

    fn add(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        self.add_with_constant(region, a, b, F::zero(), offset)
    }

    fn sub(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        self.sub_with_constant(region, a, b, F::zero(), offset)
    }

    fn add_with_constant(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        constant: F,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = match (a.value(), b.value()) {
            (Some(a), Some(b)) => Some(a + b + constant),
            _ => None,
        };

        let (_, _, res, _) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::assigned_to_add(&b),
                Term::unassigned_to_sub(c),
                Term::Zero,
            ],
            constant,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(res)
    }

    fn add_constant(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        constant: F,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = a.value().map(|a| a + constant);

        let (_, _, res, _) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::Zero,
                Term::unassigned_to_sub(c),
                Term::Zero,
            ],
            constant,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(res)
    }

    fn neg_with_constant(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        constant: F,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = a.value().map(|a| -a + constant);

        let (_, _, res, _) = self.combine(
            region,
            [
                Term::assigned_to_sub(&a),
                Term::Zero,
                Term::unassigned_to_sub(c),
                Term::Zero,
            ],
            constant,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(res)
    }

    fn sub_with_constant(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        constant: F,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = match (a.value(), b.value()) {
            (Some(a), Some(b)) => Some(a - b + constant),
            _ => None,
        };

        let (_, _, res, _) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::assigned_to_sub(&b),
                Term::unassigned_to_sub(c),
                Term::Zero,
            ],
            constant,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(res)
    }

    fn sub_sub_with_constant(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b_0: impl Assigned<F>,
        b_1: impl Assigned<F>,
        constant: F,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = match (a.value(), b_0.value(), b_1.value()) {
            (Some(a), Some(b_0), Some(b_1)) => Some(a - b_0 - b_1 + constant),
            _ => None,
        };

        let (_, _, _, res) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::assigned_to_sub(&b_0),
                Term::assigned_to_sub(&b_1),
                Term::unassigned_to_sub(c),
            ],
            constant,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(res)
    }

    fn mul2(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = a.value().map(|a| a + a);

        let (_, _, res, _) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::assigned_to_add(&a),
                Term::unassigned_to_sub(c),
                Term::Zero,
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(res)
    }

    fn mul3(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = a.value().map(|a| a + a + a);

        let (_, _, _, res) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::assigned_to_add(&a),
                Term::assigned_to_add(&a),
                Term::unassigned_to_sub(c),
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(res)
    }

    fn mul(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = match (a.value(), b.value()) {
            (Some(a), Some(b)) => Some(a * b),
            _ => None,
        };

        let (_, _, res, _) = self.combine(
            region,
            [
                Term::assigned_to_mul(&a),
                Term::assigned_to_mul(&b),
                Term::unassigned_to_sub(c),
                Term::Zero,
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(res)
    }

    fn div_unsafe(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let c = match (a.value(), b.value()) {
            (Some(a), Some(b)) => match b.invert().into() {
                Some(b_inverted) => Some(a * &b_inverted),
                // Non inversion case will never be verified
                _ => Some(F::zero()),
            },
            _ => None,
        };

        let (_, res, _, _) = self.combine(
            region,
            [
                Term::assigned_to_mul(&b),
                Term::unassigned_to_mul(c),
                Term::assigned_to_add(&a),
                Term::Zero,
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(res)
    }

    fn div(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(AssignedValue<F>, AssignedCondition<F>), Error> {
        let (b_inverted, cond) = self.invert(region, b, offset)?;
        let res = self.mul(region, a, b_inverted, offset)?;
        Ok((res, cond))
    }

    fn invert_unsafe(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        // Just enforce the equation below
        // If input 'a' is zero then no valid witness will be found
        // a * a' - 1 = 0

        let inverse = match a.value() {
            Some(a) => match a.invert().into() {
                Some(a) => Some(a),
                // Non inversion case will never be verified
                _ => Some(F::zero()),
            },
            _ => None,
        };

        let (_, inverse, _, _) = self.combine(
            region,
            [
                Term::assigned_to_mul(&a),
                Term::unassigned_to_mul(inverse),
                Term::Zero,
                Term::Zero,
            ],
            -F::one(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(inverse)
    }

    fn invert(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(AssignedValue<F>, AssignedCondition<F>), Error> {
        let (one, zero) = (F::one(), F::zero());

        // Returns 'r' as a condition bit that defines if inversion successful or not

        // First enfoce 'r' to be a bit
        // (a * a') - 1 + r = 0
        // r * a' - r = 0
        // if r = 1 then a' = 1
        // if r = 0 then a' = 1/a

        // Witness layout:
        // | A | B  | C | D |
        // | - | -- | - | - |
        // | a | a' | r | - |
        // | r | a' | r | - |

        let (r, a_inv) = match a.value() {
            Some(a) => match a.invert().into() {
                Some(e) => (Some(zero), Some(e)),
                None => (Some(one), Some(one)),
            },
            _ => (None, None),
        };

        let r = self.assign_bit(region, &r.into(), offset)?;

        // (a * a') - 1 + r = 0
        // | A | B  | C | D |
        // | - | -- | - | - |
        // | a | a' | r | - |
        let (_, a_inv, _, _) = self.combine(
            region,
            [
                Term::assigned_to_mul(&a),
                Term::unassigned_to_mul(a_inv),
                Term::assigned_to_add(&r),
                Term::Zero,
            ],
            -one,
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        // r * a' - r = 0
        // | A | B  | C | D |
        // | - | -- | - | - |
        // | r | a' | r | - |

        self.combine(
            region,
            [
                Term::assigned_to_mul(&r),
                Term::assigned_to_mul(&a_inv),
                Term::assigned_to_sub(&r),
                Term::Zero,
            ],
            zero,
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok((a_inv, r))
    }

    fn assert_equal(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::assigned_to_sub(&b),
                Term::Zero,
                Term::Zero,
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(())
    }

    fn assert_not_equal(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        // (a - b) must have an inverse
        let c = self.sub_with_constant(region, a, b, F::zero(), offset)?;
        self.assert_not_zero(region, c, offset)
    }

    fn is_equal(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedCondition<F>, Error> {
        let (one, zero) = (F::one(), F::zero());

        // Given a and b equation below is enforced
        // 0 = (a - b) * (r * (1 - x) + x) + r - 1
        // Where r and x is witnesses and r is enforced to be a bit

        // Witness layout:
        // | A   | B | C | D |
        // | --- | - | - | - |
        // | dif | a | b | - |
        // | r   | x | u | - |
        // | dif | u | r | - |

        let (x, r) = match (a.value(), b.value()) {
            (Some(a), Some(b)) => {
                let c = a - b;
                match c.invert().into() {
                    Some(inverted) => (Some(inverted), Some(zero)),
                    None => (Some(one), Some(one)),
                }
            }
            _ => (None, None),
        };

        let r = self.assign_bit(region, &r.into(), offset)?;
        let dif = self.sub(region, a, b, offset)?;

        // 0 = rx - r - x + u
        // | A   | B | C | D |
        // | --- | - | - | - |
        // | r   | x | u | - |

        let u = match (r.value(), x) {
            (Some(r), Some(x)) => Some(r - r * x + x),
            _ => None,
        };

        let (_, _, u, _) = self.combine(
            region,
            [
                Term::assigned_to_sub(&r),
                Term::unassigned_to_sub(x),
                Term::unassigned_to_add(u),
                Term::Zero,
            ],
            zero,
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        // 0 = u * dif + r - 1
        // | A   | B | C | D |
        // | --- | - | - | - |
        // | dif | u | r | - |

        self.combine(
            region,
            [
                Term::assigned_to_mul(&dif),
                Term::assigned_to_mul(&u),
                Term::assigned_to_add(&r),
                Term::Zero,
            ],
            -one,
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(r)
    }

    fn assert_zero(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        self.assert_equal_to_constant(region, a, F::zero(), offset)
    }

    fn assert_not_zero(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        // Non-zero element must have an inverse
        // a * w - 1 = 0

        let w = match a.value() {
            Some(a) => match a.invert().into() {
                Some(inverted) => Some(inverted),
                // Non inversion case will never be verified
                _ => Some(F::zero()),
            },
            _ => None,
        };

        self.combine(
            region,
            [
                Term::assigned_to_mul(&a),
                Term::unassigned_to_mul(w),
                Term::Zero,
                Term::Zero,
            ],
            -F::one(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(())
    }

    fn is_zero(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<AssignedCondition<F>, Error> {
        let (_, is_zero) = self.invert(region, a, offset)?;
        Ok(is_zero)
    }

    fn assert_one(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        self.assert_equal_to_constant(region, a, F::one(), offset)
    }

    fn assert_equal_to_constant(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: F,
        offset: &mut usize,
    ) -> Result<(), Error> {
        self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::Zero,
                Term::Zero,
                Term::Zero,
            ],
            -b,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(())
    }

    fn cond_or(
        &self,
        region: &mut Region<'_, F>,
        c1: &AssignedCondition<F>,
        c2: &AssignedCondition<F>,
        offset: &mut usize,
    ) -> Result<AssignedCondition<F>, Error> {
        let c = match (c1.value(), c2.value()) {
            (Some(c1), Some(c2)) => Some(c1 + c2 - c1 * c2),
            _ => None,
        };

        let zero = F::zero();

        // c + c1 * c2 - c1 - c2 = 0
        let (_, _, c, _) = self.combine(
            region,
            [
                Term::assigned_to_sub(c1),
                Term::assigned_to_sub(c2),
                Term::unassigned_to_add(c),
                Term::Zero,
            ],
            zero,
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(c.into())
    }

    fn cond_and(
        &self,
        region: &mut Region<'_, F>,
        c1: &AssignedCondition<F>,
        c2: &AssignedCondition<F>,
        offset: &mut usize,
    ) -> Result<AssignedCondition<F>, Error> {
        let c = match (c1.value(), c2.value()) {
            (Some(c1), Some(c2)) => Some(c1 * c2),
            _ => None,
        };

        let zero = F::zero();

        let (_, _, c, _) = self.combine(
            region,
            [
                Term::assigned_to_mul(c1),
                Term::assigned_to_mul(c2),
                Term::unassigned_to_sub(c),
                Term::Zero,
            ],
            zero,
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(c.into())
    }

    fn cond_not(
        &self,
        region: &mut Region<'_, F>,
        c: &AssignedCondition<F>,
        offset: &mut usize,
    ) -> Result<AssignedCondition<F>, Error> {
        let one = F::one();

        let not_c = c.value().map(|c| one - c);

        let (_, b, _, _) = self.combine(
            region,
            [
                Term::assigned_to_add(c),
                Term::unassigned_to_add(not_c),
                Term::Zero,
                Term::Zero,
            ],
            -one,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        Ok(b.into())
    }

    fn cond_select(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        cond: &AssignedCondition<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        // We should satisfy the equation below with bit asserted condition flag
        // c (a-b) + b - res = 0

        // Witness layout:
        // | A   | B   | C | D   |
        // | --- | --- | - | --- |
        // | dif | a   | b | -   |
        // | c   | dif | b | res |

        let (dif, res) = match (a.value(), b.value(), cond.bool_value) {
            (Some(a), Some(b), Some(cond)) => {
                let dif = a - b;
                let res = if cond { a } else { b };
                (Some(dif), Some(res))
            }
            _ => (None, None),
        };

        // a - b - dif = 0
        let (_, _, _, dif) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::assigned_to_sub(&b),
                Term::Zero,
                Term::unassigned_to_sub(dif),
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        // cond * dif + b + a_or_b  = 0
        let (_, _, _, res) = self.combine(
            region,
            [
                Term::assigned_to_mul(&dif),
                Term::assigned_to_mul(cond),
                Term::assigned_to_add(&b),
                Term::unassigned_to_sub(res),
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(res)
    }

    fn cond_select_or_assign(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: F,
        cond: &AssignedCondition<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        // We should satisfy the equation below with bit asserted condition flag
        // c (a-b_constant) + b_constant - res = 0

        // Witness layout:
        // | A   | B   | C | D   |
        // | --- | --- | - | --- |
        // | dif | a   | - | -   |
        // | c   | dif | - | res |

        let (dif, res) = match (a.value(), cond.bool_value) {
            (Some(a), Some(cond)) => {
                let dif = a - b;
                let res = if cond { a } else { b };
                (Some(dif), Some(res))
            }
            _ => (None, None),
        };

        // a - b - dif = 0
        let (_, _, _, dif) = self.combine(
            region,
            [
                Term::assigned_to_add(&a),
                Term::Zero,
                Term::Zero,
                Term::unassigned_to_sub(dif),
            ],
            -b,
            offset,
            CombinationOptionCommon::OneLinerAdd.into(),
        )?;

        // cond * dif + b + a_or_b  = 0
        let (_, _, _, res) = self.combine(
            region,
            [
                Term::assigned_to_mul(&dif),
                Term::assigned_to_mul(cond),
                Term::Zero,
                Term::unassigned_to_sub(res),
            ],
            b,
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(res)
    }

    fn assign_bit(
        &self,
        region: &mut Region<'_, F>,
        bit: &UnassignedValue<F>,
        offset: &mut usize,
    ) -> Result<AssignedBit<F>, Error> {
        // val * val - val  = 0

        // Witness layout:
        // | A   | B   | C   | D |
        // | --- | --- | --- | - |
        // | val | val | val | - |

        let (a, b, c, _) = self.combine(
            region,
            [
                Term::unassigned_to_mul(bit.value()),
                Term::unassigned_to_mul(bit.value()),
                Term::unassigned_to_sub(bit.value()),
                Term::Zero,
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        region.constrain_equal(a.cell(), b.cell())?;
        region.constrain_equal(b.cell(), c.cell())?;

        Ok(c.into())
    }

    fn assert_bit(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        // val * val - val  = 0

        // Witness layout:
        // | A   | B   | C   | D |
        // | --- | --- | --- | - |
        // | val | val | val | - |

        let (a, b, c, _) = self.combine(
            region,
            [
                Term::assigned_to_mul(&a),
                Term::assigned_to_mul(&a),
                Term::assigned_to_sub(&a),
                Term::Zero,
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        region.constrain_equal(a.cell(), b.cell())?;
        region.constrain_equal(b.cell(), c.cell())?;

        Ok(())
    }

    fn one_or_one(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        // (a-1) * (b-1)  = 0

        // Witness layout:
        // | A   | B   | C   | D |
        // | --- | --- | --- | - |
        // | val | val | -   | - |

        self.combine(
            region,
            [
                Term::assigned_to_sub(&a),
                Term::assigned_to_sub(&b),
                Term::Zero,
                Term::Zero,
            ],
            F::one(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;

        Ok(())
    }

    fn combine(
        &self,
        region: &mut Region<'_, F>,
        terms: [Term<F>; WIDTH],
        constant: F,
        offset: &mut usize,
        option: CombinationOption<F>,
    ) -> Result<CombinedValues<F>, Error> {
        let (c_0, u_0) = (terms[0].coeff(), terms[0].base());
        let (c_1, u_1) = (terms[1].coeff(), terms[1].base());
        let (c_2, u_2) = (terms[2].coeff(), terms[2].base());
        let (c_3, u_3) = (terms[3].coeff(), terms[3].base());

        let cell_0 = region
            .assign_advice(
                || "coeff_0",
                self.config.a,
                *offset,
                || c_0.ok_or(Error::Synthesis),
            )?
            .cell();
        let cell_1 = region
            .assign_advice(
                || "coeff_1",
                self.config.b,
                *offset,
                || c_1.ok_or(Error::Synthesis),
            )?
            .cell();
        let cell_2 = region
            .assign_advice(
                || "coeff_2",
                self.config.c,
                *offset,
                || c_2.ok_or(Error::Synthesis),
            )?
            .cell();
        let cell_3 = region
            .assign_advice(
                || "coeff_3",
                self.config.d,
                *offset,
                || c_3.ok_or(Error::Synthesis),
            )?
            .cell();

        region.assign_fixed(|| "base_0", self.config.sa, *offset, || Ok(u_0))?;
        region.assign_fixed(|| "base_1", self.config.sb, *offset, || Ok(u_1))?;
        region.assign_fixed(|| "base_2", self.config.sc, *offset, || Ok(u_2))?;
        region.assign_fixed(|| "base_3", self.config.sd, *offset, || Ok(u_3))?;

        region.assign_fixed(
            || "s_constant",
            self.config.s_constant,
            *offset,
            || Ok(constant),
        )?;

        match option {
            CombinationOption::Common(option) => match option {
                CombinationOptionCommon::CombineToNextMul(next) => {
                    region.assign_fixed(|| "s_mul", self.config.s_mul, *offset, || Ok(F::one()))?;
                    region.assign_fixed(|| "sd_next", self.config.sd_next, *offset, || Ok(next))?;
                }
                CombinationOptionCommon::CombineToNextScaleMul(next, scale) => {
                    region.assign_fixed(|| "s_mul", self.config.s_mul, *offset, || Ok(scale))?;
                    region.assign_fixed(|| "sd_next", self.config.sd_next, *offset, || Ok(next))?;
                }
                CombinationOptionCommon::CombineToNextAdd(next) => {
                    region.assign_fixed(|| "sd_next", self.config.sd_next, *offset, || Ok(next))?;
                    region.assign_fixed(
                        || "s_mul unused",
                        self.config.s_mul,
                        *offset,
                        || Ok(F::zero()),
                    )?;
                }
                CombinationOptionCommon::OneLinerMul => {
                    region.assign_fixed(|| "s_mul", self.config.s_mul, *offset, || Ok(F::one()))?;
                    region.assign_fixed(
                        || "sd_next unused",
                        self.config.sd_next,
                        *offset,
                        || Ok(F::zero()),
                    )?;
                }
                CombinationOptionCommon::OneLinerAdd => {
                    region.assign_fixed(
                        || "sd_next unused",
                        self.config.sd_next,
                        *offset,
                        || Ok(F::zero()),
                    )?;
                    region.assign_fixed(
                        || "s_mul unused",
                        self.config.s_mul,
                        *offset,
                        || Ok(F::zero()),
                    )?;
                }
            },
        };

        terms[0].constrain_equal(region, cell_0)?;
        terms[1].constrain_equal(region, cell_1)?;
        terms[2].constrain_equal(region, cell_2)?;
        terms[3].constrain_equal(region, cell_3)?;

        *offset += 1;

        let a_0 = AssignedValue::new(cell_0, c_0);
        let a_1 = AssignedValue::new(cell_1, c_1);
        let a_2 = AssignedValue::new(cell_2, c_2);
        let a_3 = AssignedValue::new(cell_3, c_3);

        Ok((a_0, a_1, a_2, a_3))
    }

    fn assign_value(
        &self,
        region: &mut Region<'_, F>,
        unassigned: &UnassignedValue<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        self.assign_to_column(region, unassigned, MainGateColumn::A, offset)
    }

    fn assign_to_column(
        &self,
        region: &mut Region<'_, F>,
        unassigned: &UnassignedValue<F>,
        column: MainGateColumn,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let column = match column {
            MainGateColumn::A => self.config.a,
            MainGateColumn::B => self.config.b,
            MainGateColumn::C => self.config.c,
            MainGateColumn::D => self.config.d,
        };
        let cell = region.assign_advice(
            || "assign value",
            column,
            *offset,
            || unassigned.value().ok_or(Error::Synthesis),
        )?;
        // proceed to the next row
        self.no_operation(region, offset)?;

        Ok(unassigned.assign(cell.cell()))
    }

    fn assign_to_acc(
        &self,
        region: &mut Region<'_, F>,
        unassigned: &UnassignedValue<F>,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        self.assign_to_column(region, unassigned, MainGateColumn::D, offset)
    }

    fn nand(
        &self,
        region: &mut Region<'_, F>,
        a: impl Assigned<F>,
        b: impl Assigned<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        self.combine(
            region,
            [
                Term::assigned_to_mul(&a),
                Term::assigned_to_mul(&b),
                Term::Zero,
                Term::Zero,
            ],
            F::zero(),
            offset,
            CombinationOptionCommon::OneLinerMul.into(),
        )?;
        Ok(())
    }

    fn no_operation(&self, region: &mut Region<'_, F>, offset: &mut usize) -> Result<(), Error> {
        region.assign_fixed(|| "s_mul", self.config.s_mul, *offset, || Ok(F::zero()))?;
        region.assign_fixed(|| "sc", self.config.sc, *offset, || Ok(F::zero()))?;
        region.assign_fixed(|| "sa", self.config.sa, *offset, || Ok(F::zero()))?;
        region.assign_fixed(|| "sb", self.config.sb, *offset, || Ok(F::zero()))?;
        region.assign_fixed(|| "sd", self.config.sd, *offset, || Ok(F::zero()))?;
        region.assign_fixed(|| "sd_next", self.config.sd_next, *offset, || Ok(F::zero()))?;
        region.assign_fixed(
            || "s_constant",
            self.config.s_constant,
            *offset,
            || Ok(F::zero()),
        )?;
        *offset += 1;
        Ok(())
    }
}

impl<F: FieldExt> MainGate<F> {
    pub fn new(config: MainGateConfig) -> Self {
        MainGate {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> MainGateConfig {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let d = meta.advice_column();

        let sa = meta.fixed_column();
        let sb = meta.fixed_column();
        let sc = meta.fixed_column();
        let sd = meta.fixed_column();
        let sd_next = meta.fixed_column();
        let s_mul = meta.fixed_column();
        let s_constant = meta.fixed_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(d);

        meta.create_gate("main_gate", |meta| {
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            let d_next = meta.query_advice(d, Rotation::next());
            let d = meta.query_advice(d, Rotation::cur());

            let sa = meta.query_fixed(sa, Rotation::cur());
            let sb = meta.query_fixed(sb, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());
            let sd = meta.query_fixed(sd, Rotation::cur());

            let sd_next = meta.query_fixed(sd_next, Rotation::cur());

            let s_mul = meta.query_fixed(s_mul, Rotation::cur());

            let s_constant = meta.query_fixed(s_constant, Rotation::cur());

            vec![
                a.clone() * sa
                    + b.clone() * sb
                    + a * b * s_mul
                    + c * sc
                    + sd * d
                    + sd_next * d_next
                    + s_constant,
            ]
        });

        MainGateConfig {
            a,
            b,
            c,
            d,
            sa,
            sb,
            sc,
            sd,
            sd_next,
            s_constant,
            s_mul,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::marker::PhantomData;

    use super::{MainGate, MainGateConfig, Term};
    use crate::halo2::arithmetic::FieldExt;
    use crate::halo2::circuit::{Layouter, SimpleFloorPlanner};
    use crate::halo2::dev::MockProver;
    use crate::halo2::plonk::{Circuit, ConstraintSystem, Error};
    use crate::main_gate::{CombinationOptionCommon, MainGateInstructions};
    use crate::{AssignedCondition, UnassignedValue};
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    #[cfg(feature = "kzg")]
    use crate::halo2::pairing::bn256::Fr as Fp;
    #[cfg(feature = "zcash")]
    use crate::halo2::pasta::Fp;

    #[derive(Clone)]
    struct TestCircuitConfig {
        main_gate_config: MainGateConfig,
    }

    impl TestCircuitConfig {
        fn main_gate<F: FieldExt>(&self) -> MainGate<F> {
            MainGate::<F> {
                config: self.main_gate_config.clone(),
                _marker: PhantomData,
            }
        }
    }

    #[derive(Default)]
    struct TestCircuitCombination<F: FieldExt> {
        _marker: PhantomData<F>,
    }

    impl<F: FieldExt> Circuit<F> for TestCircuitCombination<F> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            let main_gate_config = MainGate::<F>::configure(meta);
            TestCircuitConfig { main_gate_config }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let main_gate = config.main_gate();

            let mut rng = XorShiftRng::from_seed([
                0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06,
                0xbc, 0xe5,
            ]);

            let mut rand = || -> F { F::random(&mut rng) };

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    // OneLinerAdd

                    let (a_0, a_1, a_2, a_3) = (rand(), rand(), rand(), rand());
                    let (r_0, r_1, r_2, r_3) = (rand(), rand(), rand(), rand());

                    let constant = -(a_0 * r_0 + a_1 * r_1 + a_2 * r_2 + a_3 * r_3);

                    let terms = [
                        Term::Unassigned(Some(a_0), r_0),
                        Term::Unassigned(Some(a_1), r_1),
                        Term::Unassigned(Some(a_2), r_2),
                        Term::Unassigned(Some(a_3), r_3),
                    ];

                    let (u_0, u_1, u_2, u_3) = main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::OneLinerAdd.into(),
                    )?;

                    let terms = [
                        Term::Assigned(&u_0, r_0),
                        Term::Assigned(&u_1, r_1),
                        Term::Assigned(&u_2, r_2),
                        Term::Assigned(&u_3, r_3),
                    ];

                    main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::OneLinerAdd.into(),
                    )?;

                    // OneLinerMul

                    let (a_0, a_1, a_2, a_3) = (rand(), rand(), rand(), rand());
                    let (r_0, r_1, r_2, r_3) = (rand(), rand(), rand(), rand());

                    let constant = -(a_0 * a_1 + a_0 * r_0 + a_1 * r_1 + a_2 * r_2 + a_3 * r_3);

                    let terms = [
                        Term::Unassigned(Some(a_0), r_0),
                        Term::Unassigned(Some(a_1), r_1),
                        Term::Unassigned(Some(a_2), r_2),
                        Term::Unassigned(Some(a_3), r_3),
                    ];

                    let (u_0, u_1, u_2, u_3) = main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::OneLinerMul.into(),
                    )?;

                    let terms = [
                        Term::Assigned(&u_0, r_0),
                        Term::Assigned(&u_1, r_1),
                        Term::Assigned(&u_2, r_2),
                        Term::Assigned(&u_3, r_3),
                    ];

                    main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::OneLinerMul.into(),
                    )?;

                    // CombineToNextMul(F)

                    let (a_0, a_1, a_2, a_3, a_next) = (rand(), rand(), rand(), rand(), rand());
                    let (r_0, r_1, r_2, r_3, r_next) = (rand(), rand(), rand(), rand(), rand());

                    let constant = -(a_0 * a_1
                        + r_0 * a_0
                        + r_1 * a_1
                        + a_2 * r_2
                        + a_3 * r_3
                        + a_next * r_next);

                    let terms = [
                        Term::Unassigned(Some(a_0), r_0),
                        Term::Unassigned(Some(a_1), r_1),
                        Term::Unassigned(Some(a_2), r_2),
                        Term::Unassigned(Some(a_3), r_3),
                    ];

                    let (u_0, u_1, u_2, u_3) = main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::CombineToNextMul(r_next).into(),
                    )?;

                    main_gate.assign_to_acc(&mut region, &Some(a_next).into(), offset)?;

                    let terms = [
                        Term::Assigned(&u_0, r_0),
                        Term::Assigned(&u_1, r_1),
                        Term::Assigned(&u_2, r_2),
                        Term::Assigned(&u_3, r_3),
                    ];

                    main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::CombineToNextMul(r_next).into(),
                    )?;

                    main_gate.assign_to_acc(&mut region, &Some(a_next).into(), offset)?;

                    // CombineToNextScaleMul(F, F)

                    let (a_0, a_1, a_2, a_3, a_next) = (rand(), rand(), rand(), rand(), rand());
                    let (r_scale, r_0, r_1, r_2, r_3, r_next) =
                        (rand(), rand(), rand(), rand(), rand(), rand());

                    let constant = -(r_scale * a_0 * a_1
                        + r_0 * a_0
                        + r_1 * a_1
                        + a_2 * r_2
                        + a_3 * r_3
                        + a_next * r_next);

                    let terms = [
                        Term::Unassigned(Some(a_0), r_0),
                        Term::Unassigned(Some(a_1), r_1),
                        Term::Unassigned(Some(a_2), r_2),
                        Term::Unassigned(Some(a_3), r_3),
                    ];

                    let (u_0, u_1, u_2, u_3) = main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::CombineToNextScaleMul(r_next, r_scale).into(),
                    )?;

                    main_gate.assign_to_acc(&mut region, &Some(a_next).into(), offset)?;

                    let terms = [
                        Term::Assigned(&u_0, r_0),
                        Term::Assigned(&u_1, r_1),
                        Term::Assigned(&u_2, r_2),
                        Term::Assigned(&u_3, r_3),
                    ];

                    main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::CombineToNextScaleMul(r_next, r_scale).into(),
                    )?;

                    main_gate.assign_to_acc(&mut region, &Some(a_next).into(), offset)?;

                    // CombineToNextAdd(F)

                    let (a_0, a_1, a_2, a_3, a_next) = (rand(), rand(), rand(), rand(), rand());
                    let (r_0, r_1, r_2, r_3, r_next) = (rand(), rand(), rand(), rand(), rand());

                    let constant =
                        -(r_0 * a_0 + r_1 * a_1 + a_2 * r_2 + a_3 * r_3 + a_next * r_next);

                    let terms = [
                        Term::Unassigned(Some(a_0), r_0),
                        Term::Unassigned(Some(a_1), r_1),
                        Term::Unassigned(Some(a_2), r_2),
                        Term::Unassigned(Some(a_3), r_3),
                    ];

                    let (u_0, u_1, u_2, u_3) = main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::CombineToNextAdd(r_next).into(),
                    )?;

                    main_gate.assign_to_acc(&mut region, &Some(a_next).into(), offset)?;

                    let terms = [
                        Term::Assigned(&u_0, r_0),
                        Term::Assigned(&u_1, r_1),
                        Term::Assigned(&u_2, r_2),
                        Term::Assigned(&u_3, r_3),
                    ];

                    main_gate.combine(
                        &mut region,
                        terms,
                        constant,
                        offset,
                        CombinationOptionCommon::CombineToNextAdd(r_next).into(),
                    )?;

                    main_gate.assign_to_acc(&mut region, &Some(a_next).into(), offset)?;

                    Ok(())
                },
            )?;

            Ok(())
        }
    }

    #[test]
    fn test_main_gate_combination() {
        const K: u32 = 8;
        let circuit = TestCircuitCombination::<Fp> {
            _marker: PhantomData,
        };
        let prover = match MockProver::run(K, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default)]
    struct TestCircuitBitness<F: FieldExt> {
        neg_path: bool,
        _marker: PhantomData<F>,
    }

    impl<F: FieldExt> Circuit<F> for TestCircuitBitness<F> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            let main_gate_config = MainGate::<F>::configure(meta);
            TestCircuitConfig { main_gate_config }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let main_gate = config.main_gate();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    if self.neg_path {
                        let minus_one = -F::one();
                        main_gate.assign_bit(
                            &mut region,
                            &UnassignedValue(Some(minus_one)),
                            offset,
                        )?;
                    } else {
                        let one = F::one();
                        let zero = F::zero();

                        let u = main_gate.assign_bit(
                            &mut region,
                            &UnassignedValue(Some(one)),
                            offset,
                        )?;
                        main_gate.assert_bit(&mut region, u, offset)?;

                        let u = main_gate.assign_bit(
                            &mut region,
                            &UnassignedValue(Some(zero)),
                            offset,
                        )?;
                        main_gate.assert_bit(&mut region, u, offset)?;
                    }

                    Ok(())
                },
            )?;

            Ok(())
        }
    }

    #[test]
    fn test_main_gate_bitness() {
        const K: u32 = 8;
        let circuit = TestCircuitBitness::<Fp> {
            neg_path: false,
            _marker: PhantomData,
        };
        let prover = match MockProver::run(K, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));

        let circuit = TestCircuitBitness::<Fp> {
            neg_path: true,
            _marker: PhantomData,
        };
        let prover = match MockProver::run(K, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_ne!(prover.verify(), Ok(()));
    }

    #[derive(Default)]
    struct TestCircuitEquality<F: FieldExt> {
        neg_path: bool,
        _marker: PhantomData<F>,
    }

    impl<F: FieldExt> Circuit<F> for TestCircuitEquality<F> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            let main_gate_config = MainGate::<F>::configure(meta);
            TestCircuitConfig { main_gate_config }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let main_gate = config.main_gate();

            let mut rng = XorShiftRng::from_seed([
                0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06,
                0xbc, 0xe5,
            ]);

            let mut rand = || -> F { F::random(&mut rng) };

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let offset = &mut 0;

                    if self.neg_path {
                    } else {
                        let one = F::one();
                        let zero = F::zero();

                        let assigned_one =
                            &main_gate.assign_bit(&mut region, &Some(one).into(), offset)?;

                        let assigned_zero =
                            &main_gate.assign_bit(&mut region, &Some(zero).into(), offset)?;

                        // assert_equal_to_constant

                        let val = rand();
                        let assigned =
                            &main_gate.assign_value(&mut region, &Some(val).into(), offset)?;
                        main_gate.assert_equal_to_constant(&mut region, assigned, val, offset)?;
                        main_gate.assert_not_zero(&mut region, assigned, offset)?;

                        // assert_equal

                        let val = rand();
                        let assigned_0 =
                            main_gate.assign_value(&mut region, &Some(val).into(), offset)?;
                        let assigned_1 =
                            main_gate.assign_value(&mut region, &Some(val).into(), offset)?;
                        main_gate.assert_equal(&mut region, assigned_0, assigned_1, offset)?;

                        // assert_not_equal

                        let val_0 = rand();
                        let val_1 = rand();
                        let assigned_0 =
                            main_gate.assign_value(&mut region, &Some(val_0).into(), offset)?;
                        let assigned_1 =
                            main_gate.assign_value(&mut region, &Some(val_1).into(), offset)?;
                        main_gate.assert_not_equal(&mut region, assigned_0, assigned_1, offset)?;

                        // is_equal

                        let val = rand();
                        let assigned_0 =
                            main_gate.assign_value(&mut region, &Some(val).into(), offset)?;
                        let assigned_1 =
                            main_gate.assign_value(&mut region, &Some(val).into(), offset)?;
                        let is_equal =
                            &main_gate.is_equal(&mut region, assigned_0, assigned_1, offset)?;

                        main_gate.assert_one(&mut region, is_equal, offset)?;
                        main_gate.assert_equal(&mut region, is_equal, assigned_one, offset)?;

                        let val_0 = rand();
                        let val_1 = rand();
                        let assigned_0 =
                            main_gate.assign_value(&mut region, &Some(val_0).into(), offset)?;
                        let assigned_1 =
                            main_gate.assign_value(&mut region, &Some(val_1).into(), offset)?;
                        let is_equal =
                            &main_gate.is_equal(&mut region, assigned_0, assigned_1, offset)?;

                        main_gate.assert_zero(&mut region, is_equal, offset)?;
                        main_gate.assert_equal(&mut region, is_equal, assigned_zero, offset)?;

                        // is_zero

                        let val = rand();
                        let assigned =
                            main_gate.assign_value(&mut region, &Some(val).into(), offset)?;
                        let is_zero = &main_gate.is_zero(&mut region, assigned, offset)?;
                        main_gate.assert_zero(&mut region, is_zero, offset)?;
                        main_gate.assert_equal(&mut region, is_zero, assigned_zero, offset)?;

                        let is_zero = &main_gate.is_zero(&mut region, assigned_zero, offset)?;
                        main_gate.assert_one(&mut region, is_zero, offset)?;
                        main_gate.assert_equal(&mut region, is_zero, assigned_one, offset)?;
                    }

                    Ok(())
                },
            )?;

            Ok(())
        }
    }

    #[test]
    fn test_main_gate_equaility() {
        const K: u32 = 8;
        let circuit = TestCircuitEquality::<Fp> {
            neg_path: false,
            _marker: PhantomData,
        };
        let prover = match MockProver::run(K, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));

        // let circuit = TestCircuitBitness::<Fp> {
        //     neg_path: true,
        //     _marker: PhantomData,
        // };
        // let prover = match MockProver::run(K, &circuit, vec![]) {
        //     Ok(prover) => prover,
        //     Err(e) => panic!("{:#?}", e),
        // };
        // assert_ne!(prover.verify(), Ok(()));
    }

    #[derive(Default)]
    struct TestCircuitArith<F: FieldExt> {
        _marker: PhantomData<F>,
    }

    impl<F: FieldExt> Circuit<F> for TestCircuitArith<F> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            let main_gate_config = MainGate::<F>::configure(meta);
            TestCircuitConfig { main_gate_config }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let main_gate = config.main_gate();

            let mut rng = XorShiftRng::from_seed([
                0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06,
                0xbc, 0xe5,
            ]);

            let mut rand = || -> F { F::random(&mut rng) };

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let mut offset = 0;

                    let a = rand();
                    let b = rand();
                    let c = a + b;
                    let a = Some(a);
                    let b = Some(b);
                    let c = Some(c);

                    let a = main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let c_0 = main_gate.assign_value(&mut region, &c.into(), &mut offset)?;
                    let c_1 = main_gate.add(&mut region, a, b, &mut offset)?;
                    main_gate.assert_equal(&mut region, c_0, c_1, &mut offset)?;

                    let a = rand();
                    let b = rand();
                    let c = a + b;
                    let a = Some(a);
                    let c = Some(c);

                    let a = main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let c_0 = main_gate.assign_value(&mut region, &c.into(), &mut offset)?;
                    let c_1 = main_gate.add_constant(&mut region, a, b, &mut offset)?;
                    main_gate.assert_equal(&mut region, c_0, c_1, &mut offset)?;

                    let a = rand();
                    let b = rand();
                    let constant = rand();
                    let c = a + b + constant;
                    let a = Some(a);
                    let b = Some(b);
                    let c = Some(c);

                    let a = main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let c_0 = main_gate.assign_value(&mut region, &c.into(), &mut offset)?;
                    let c_1 =
                        main_gate.add_with_constant(&mut region, a, b, constant, &mut offset)?;
                    main_gate.assert_equal(&mut region, c_0, c_1, &mut offset)?;

                    let a = rand();
                    let b = rand();
                    let c = a - b;
                    let a = Some(a);
                    let b = Some(b);
                    let c = Some(c);

                    let a = main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let c_0 = main_gate.assign_value(&mut region, &c.into(), &mut offset)?;
                    let c_1 = main_gate.sub(&mut region, a, b, &mut offset)?;
                    main_gate.assert_equal(&mut region, c_0, c_1, &mut offset)?;

                    let a = rand();
                    let b = rand();
                    let constant = rand();
                    let c = a - b + constant;
                    let a = Some(a);
                    let b = Some(b);
                    let c = Some(c);

                    let a = main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let c_0 = main_gate.assign_value(&mut region, &c.into(), &mut offset)?;
                    let c_1 =
                        main_gate.sub_with_constant(&mut region, a, b, constant, &mut offset)?;
                    main_gate.assert_equal(&mut region, c_0, c_1, &mut offset)?;

                    let a = rand();
                    let b = rand();
                    let c = a * b;
                    let a = Some(a);
                    let b = Some(b);
                    let c = Some(c);

                    let a = main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let c_0 = main_gate.assign_value(&mut region, &c.into(), &mut offset)?;
                    let c_1 = main_gate.mul(&mut region, a, b, &mut offset)?;
                    main_gate.assert_equal(&mut region, c_0, c_1, &mut offset)?;

                    let a = rand();
                    let b = rand();
                    let c = a * b.invert().unwrap();
                    let a = Some(a);
                    let b = Some(b);
                    let c = Some(c);

                    let a = main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let c_0 = main_gate.assign_value(&mut region, &c.into(), &mut offset)?;
                    let (c_1, _) = main_gate.div(&mut region, a, b, &mut offset)?;
                    main_gate.assert_equal(&mut region, c_0, c_1, &mut offset)?;

                    Ok(())
                },
            )?;

            Ok(())
        }
    }

    #[test]
    fn test_main_gate_arith() {
        const K: u32 = 8;

        let circuit = TestCircuitArith::<Fp> {
            _marker: PhantomData::<Fp>,
        };

        let prover = match MockProver::run(K, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };

        assert_eq!(prover.verify(), Ok(()));
    }

    #[derive(Default)]
    struct TestCircuitConditionals<F: FieldExt> {
        _marker: PhantomData<F>,
    }

    impl<F: FieldExt> Circuit<F> for TestCircuitConditionals<F> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            let main_gate_config = MainGate::<F>::configure(meta);
            TestCircuitConfig { main_gate_config }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let main_gate = config.main_gate();

            let mut rng = XorShiftRng::from_seed([
                0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06,
                0xbc, 0xe5,
            ]);

            let mut rand = || -> F { F::random(&mut rng) };

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let mut offset = 0;

                    let a = rand();
                    let b = rand();
                    let cond = F::zero();

                    let a = Some(a);
                    let b = Some(b);
                    let cond = Some(cond);

                    let a = &main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = &main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let cond: AssignedCondition<F> = main_gate
                        .assign_value(&mut region, &cond.into(), &mut offset)?
                        .into();
                    let selected = main_gate.cond_select(&mut region, a, b, &cond, &mut offset)?;
                    main_gate.assert_equal(&mut region, b, selected, &mut offset)?;

                    let a = rand();
                    let b = rand();
                    let cond = F::one();

                    let a = Some(a);
                    let b = Some(b);
                    let cond = Some(cond);

                    let a = &main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b = &main_gate.assign_value(&mut region, &b.into(), &mut offset)?;
                    let cond: AssignedCondition<F> = main_gate
                        .assign_value(&mut region, &cond.into(), &mut offset)?
                        .into();
                    let selected = main_gate.cond_select(&mut region, a, b, &cond, &mut offset)?;
                    main_gate.assert_equal(&mut region, a, selected, &mut offset)?;

                    let a = rand();
                    let b_constant = rand();
                    let cond = F::zero();

                    let a = Some(a);
                    let b_unassigned = Some(b_constant);
                    let cond = Some(cond);

                    let a = &main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let b_assigned =
                        &main_gate.assign_value(&mut region, &b_unassigned.into(), &mut offset)?;
                    let cond: AssignedCondition<F> = main_gate
                        .assign_value(&mut region, &cond.into(), &mut offset)?
                        .into();
                    let selected = main_gate.cond_select_or_assign(
                        &mut region,
                        a,
                        b_constant,
                        &cond,
                        &mut offset,
                    )?;
                    main_gate.assert_equal(&mut region, b_assigned, selected, &mut offset)?;

                    let a = rand();
                    let b_constant = rand();
                    let cond = F::one();

                    let a = Some(a);
                    let cond = Some(cond);

                    let a = &main_gate.assign_value(&mut region, &a.into(), &mut offset)?;
                    let cond: AssignedCondition<F> = main_gate
                        .assign_value(&mut region, &cond.into(), &mut offset)?
                        .into();
                    let selected = main_gate.cond_select_or_assign(
                        &mut region,
                        a,
                        b_constant,
                        &cond,
                        &mut offset,
                    )?;
                    main_gate.assert_equal(&mut region, a, selected, &mut offset)?;

                    Ok(())
                },
            )?;

            Ok(())
        }
    }

    #[test]
    fn test_main_gate_cond() {
        const K: u32 = 8;

        let circuit = TestCircuitConditionals::<Fp> {
            _marker: PhantomData::<Fp>,
        };

        let prover = match MockProver::run(K, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };

        assert_eq!(prover.verify(), Ok(()));
    }
}
