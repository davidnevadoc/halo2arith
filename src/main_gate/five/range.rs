use super::main_gate::{MainGate, MainGateColumn, MainGateConfig};
use super::NUMBER_OF_LOOKUP_LIMBS;
use crate::halo2::arithmetic::FieldExt;
use crate::halo2::circuit::{Chip, Layouter, Region};
use crate::halo2::plonk::{ConstraintSystem, Error, Selector, TableColumn};
use crate::halo2::poly::Rotation;
use crate::main_gate::{CombinationOptionCommon, MainGateInstructions, Term};
use crate::{AssignedValue, UnassignedValue};

#[cfg(not(feature = "no_lookup"))]
#[derive(Clone, Debug)]
pub struct TableConfig {
    selector: Selector,
    column: TableColumn,
    bit_len: usize,
}

#[derive(Clone, Debug)]
pub struct RangeConfig {
    main_gate_config: MainGateConfig,

    #[cfg(not(feature = "no_lookup"))]
    s_dense_limb_range: Selector,

    #[cfg(not(feature = "no_lookup"))]
    dense_limb_range_table: TableColumn,

    #[cfg(not(feature = "no_lookup"))]
    fine_tune_tables: Vec<TableConfig>,
}

pub struct RangeChip<F: FieldExt> {
    config: RangeConfig,
    base_bit_len: usize,
    left_shifter: Vec<F>,
}

impl<F: FieldExt> RangeChip<F> {
    #[cfg(not(feature = "no_lookup"))]
    fn get_table(&self, bit_len: usize) -> Result<&TableConfig, Error> {
        let table_config = self
            .config
            .fine_tune_tables
            .iter()
            .find(|&table_config| table_config.bit_len == bit_len)
            .ok_or(Error::Synthesis)?;
        Ok(table_config)
    }

    fn main_gate_config(&self) -> MainGateConfig {
        self.config.main_gate_config.clone()
    }

    fn main_gate(&self) -> MainGate<F> {
        MainGate::<F>::new(self.main_gate_config())
    }
}

impl<F: FieldExt> Chip<F> for RangeChip<F> {
    type Config = RangeConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config {
        &self.config
    }
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

pub trait RangeInstructions<F: FieldExt>: Chip<F> {
    fn range_value(
        &self,
        region: &mut Region<'_, F>,
        input: &UnassignedValue<F>,
        bit_len: usize,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error>;

    #[cfg(not(feature = "no_lookup"))]
    fn load_limb_range_table(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error>;
    #[cfg(not(feature = "no_lookup"))]
    fn load_overflow_range_tables(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error>;
}

impl<F: FieldExt> RangeInstructions<F> for RangeChip<F> {
    fn range_value(
        &self,
        region: &mut Region<'_, F>,
        input: &UnassignedValue<F>,
        bit_len: usize,
        offset: &mut usize,
    ) -> Result<AssignedValue<F>, Error> {
        let main_gate = self.main_gate();
        let (one, zero) = (F::one(), F::zero());
        let r = self.left_shifter[0];
        let rr = self.left_shifter[1];
        let rrr = self.left_shifter[2];
        let rrrr = self.left_shifter[3];

        let number_of_dense_limbs = bit_len / self.base_bit_len;
        let fine_limb_bit_len = bit_len % self.base_bit_len;
        let number_of_limbs = number_of_dense_limbs + if fine_limb_bit_len == 0 { 0 } else { 1 };

        assert!(number_of_dense_limbs < NUMBER_OF_LOOKUP_LIMBS + 1);
        assert!(number_of_limbs > 0);

        if number_of_dense_limbs != 0 {
            // Enable dense decomposion range check.
            // Notice that fine tune limb will be in the dense limb set.
            #[cfg(not(feature = "no_lookup"))]
            self.config.s_dense_limb_range.enable(region, *offset)?;
        }

        // Bases for linear combination to the input.
        let bases = vec![one, r, rr, rrr, rrrr];

        if (number_of_dense_limbs == 1 && fine_limb_bit_len == 0) || number_of_dense_limbs == 0 {
            // Single row range proof case
            // Only assign the input to this row

            // Open small table selector if this value is in small table
            if number_of_dense_limbs == 0 {
                #[cfg(not(feature = "no_lookup"))]
                self.get_table(fine_limb_bit_len)?
                    .selector
                    .enable(region, *offset)?;
            }

            // | A   | B   | C   | D   | E   |
            // | --- | --- | --- | --- | --- |
            // | -   | a_0 | -   | -   | -   |
            main_gate.assign_to_column(region, input, MainGateColumn::B, offset)
        } else {
            let first_row_with_fine_tune = number_of_dense_limbs < 4 && fine_limb_bit_len != 0;
            let has_overflow = number_of_dense_limbs == 4 && fine_limb_bit_len > 0;

            // Enable table selector for last limb ie fine tuning limb.
            if first_row_with_fine_tune {
                #[cfg(not(feature = "no_lookup"))]
                self.get_table(fine_limb_bit_len)?
                    .selector
                    .enable(region, *offset)?;
            }

            // Input is decomposed insto smaller limbs
            let limbs = input.decompose(number_of_limbs, self.base_bit_len);

            // Witness layouts for different cases:

            // number_of_dense_limbs = 4 & fine_limb_len = 0 or
            // number_of_dense_limbs = 3 & fine_limb_len > 0
            // | A   | B   | C   | D   | E   |
            // | --- | --- | --- | --- | --- |
            // | a_0 | a_3 | a_1 | a_2 | in  |

            // number_of_dense_limbs = 3 & fine_limb_len = 0 or
            // number_of_dense_limbs = 2 & fine_limb_len > 0
            // | A   | B   | C   | D   | E   |
            // | --- | --- | --- | --- | --- |
            // | a_0 | a_2 | a_1 | -   | in  |

            // number_of_dense_limbs = 2 & fine_limb_len = 0 or
            // number_of_dense_limbs = 1 & fine_limb_len > 0
            // | A   | B   | C   | D   | E   |
            // | --- | --- | --- | --- | --- |
            // | a_0 | a_1 | -   | -   | in  |

            // number_of_dense_limbs = 4 & fine_limb_len > 1
            // | A   | B   | C   | D   | E   |
            // | --- | --- | --- | --- | --- |
            // | a_0 | a_3 | a_1 | a_2 | -   |
            // | -   | a_4 | -   | in  | t  |

            // Least significant Term in first row
            let term_0 = Term::Unassigned(limbs.as_ref().map(|limbs| limbs[0]), bases[0]);

            // Most significant Term in first row
            let term_1 = if has_overflow {
                Term::Unassigned(
                    limbs.as_ref().map(|limbs| limbs[number_of_limbs - 2]),
                    bases[number_of_limbs - 2],
                )
            } else {
                Term::Unassigned(
                    limbs.as_ref().map(|limbs| limbs[number_of_limbs - 1]),
                    bases[number_of_limbs - 1],
                )
            };

            let term_2 = if number_of_limbs > 2 {
                Term::Unassigned(limbs.as_ref().map(|limbs| limbs[1]), bases[1])
            } else {
                Term::Zero
            };

            let term_3 = if number_of_limbs > 3 {
                Term::Unassigned(limbs.as_ref().map(|limbs| limbs[2]), bases[2])
            } else {
                Term::Zero
            };

            if has_overflow {
                let _ = main_gate.combine(
                    region,
                    [term_0, term_1, term_2, term_3, Term::Zero],
                    zero,
                    offset,
                    CombinationOptionCommon::CombineToNextAdd(-one).into(),
                )?;

                assert!(number_of_limbs - 1 == 4);
                let unassigned_input = Term::Unassigned(input.value(), -one);
                let (intermediate, overflow) = match limbs {
                    Some(limbs) => {
                        let overflow = limbs[4];
                        // input value must exist if limbs do
                        let input_value = input.value().unwrap();
                        // combination of previous row must go to column 'E'
                        let intermediate = input_value - overflow * rrrr;
                        (Some(intermediate), Some(overflow))
                    }
                    None => (None, None),
                };
                let intermediate = Term::Unassigned(intermediate, one);
                let overflow = Term::Unassigned(overflow, rrrr);

                // should meet with overflow bit len
                #[cfg(not(feature = "no_lookup"))]
                self.get_table(fine_limb_bit_len)?
                    .selector
                    .enable(region, *offset)?;

                let (_, _, _, assigned, _) = main_gate.combine(
                    region,
                    [
                        Term::Zero,
                        overflow,
                        Term::Zero,
                        unassigned_input,
                        intermediate,
                    ],
                    zero,
                    offset,
                    CombinationOptionCommon::OneLinerAdd.into(),
                )?;

                Ok(assigned)
            } else {
                let unassigned_input = Term::Unassigned(input.value(), -one);
                let combination_option = CombinationOptionCommon::OneLinerAdd.into();
                let (_, _, _, _, assigned) = main_gate.combine(
                    region,
                    [term_0, term_1, term_2, term_3, unassigned_input],
                    zero,
                    offset,
                    combination_option,
                )?;
                Ok(assigned)
            }
        }
    }

    #[cfg(not(feature = "no_lookup"))]
    fn load_limb_range_table(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        let table_values: Vec<F> = (0..1 << self.base_bit_len).map(|e| F::from(e)).collect();

        layouter.assign_table(
            || "",
            |mut table| {
                for (index, &value) in table_values.iter().enumerate() {
                    table.assign_cell(
                        || "limb range table",
                        self.config.dense_limb_range_table,
                        index,
                        || Ok(value),
                    )?;
                }
                Ok(())
            },
        )?;
        Ok(())
    }

    #[cfg(not(feature = "no_lookup"))]
    fn load_overflow_range_tables(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        for overflow_table in self.config.fine_tune_tables.iter() {
            let bit_len = overflow_table.bit_len;
            let table_values: Vec<F> = (0..1 << bit_len).map(|e| F::from(e)).collect();

            layouter.assign_table(
                || "",
                |mut table| {
                    for (index, &value) in table_values.iter().enumerate() {
                        table.assign_cell(
                            || "overflow table",
                            overflow_table.column,
                            index,
                            || Ok(value),
                        )?;
                    }
                    Ok(())
                },
            )?;
        }

        Ok(())
    }
}

impl<F: FieldExt> RangeChip<F> {
    pub fn new(config: RangeConfig, base_bit_len: usize) -> Self {
        let two = F::from(2);
        let left_shifter_r = two.pow(&[base_bit_len as u64, 0, 0, 0]);
        let left_shifter_2r = two.pow(&[(base_bit_len * 2) as u64, 0, 0, 0]);
        let left_shifter_3r = two.pow(&[(base_bit_len * 3) as u64, 0, 0, 0]);
        let left_shifter_4r = two.pow(&[(base_bit_len * 4) as u64, 0, 0, 0]);

        RangeChip {
            config,
            base_bit_len,
            left_shifter: vec![
                left_shifter_r,
                left_shifter_2r,
                left_shifter_3r,
                left_shifter_4r,
            ],
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        main_gate_config: &MainGateConfig,
        fine_tune_bit_lengths: Vec<usize>,
    ) -> RangeConfig {
        let mut fine_tune_bit_lengths = fine_tune_bit_lengths;
        fine_tune_bit_lengths.sort();
        fine_tune_bit_lengths.dedup();
        let fine_tune_bit_lengths: Vec<usize> = fine_tune_bit_lengths
            .into_iter()
            .filter(|e| *e != 0)
            .collect();

        let a = main_gate_config.a;
        let b = main_gate_config.b;
        let c = main_gate_config.c;
        let d = main_gate_config.d;

        #[cfg(not(feature = "no_lookup"))]
        let s_dense_limb_range = meta.complex_selector();
        #[cfg(not(feature = "no_lookup"))]
        let dense_limb_range_table = meta.lookup_table_column();

        #[cfg(not(feature = "no_lookup"))]
        {
            meta.lookup(|meta| {
                let exp = meta.query_advice(a, Rotation::cur());
                let s_range = meta.query_selector(s_dense_limb_range);
                vec![(exp * s_range, dense_limb_range_table)]
            });

            meta.lookup(|meta| {
                let exp = meta.query_advice(b, Rotation::cur());
                let s_range = meta.query_selector(s_dense_limb_range);
                vec![(exp * s_range, dense_limb_range_table)]
            });

            meta.lookup(|meta| {
                let exp = meta.query_advice(c, Rotation::cur());
                let s_range = meta.query_selector(s_dense_limb_range);
                vec![(exp * s_range, dense_limb_range_table)]
            });

            meta.lookup(|meta| {
                let exp = meta.query_advice(d, Rotation::cur());
                let s_range = meta.query_selector(s_dense_limb_range);
                vec![(exp * s_range, dense_limb_range_table)]
            });
        }

        #[cfg(not(feature = "no_lookup"))]
        let fine_tune_tables = fine_tune_bit_lengths
            .iter()
            .map(|bit_len| {
                let selector = meta.complex_selector();
                let column = meta.lookup_table_column();

                meta.lookup(|meta| {
                    let exp = meta.query_advice(b, Rotation::cur());
                    let selector = meta.query_selector(selector);
                    vec![(exp * selector, column)]
                });

                TableConfig {
                    selector,
                    column,
                    bit_len: *bit_len,
                }
            })
            .collect();

        RangeConfig {
            main_gate_config: main_gate_config.clone(),
            #[cfg(not(feature = "no_lookup"))]
            s_dense_limb_range,
            #[cfg(not(feature = "no_lookup"))]
            dense_limb_range_table,
            #[cfg(not(feature = "no_lookup"))]
            fine_tune_tables,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{RangeChip, RangeConfig, RangeInstructions};
    use crate::halo2::arithmetic::FieldExt;
    use crate::halo2::circuit::{Layouter, SimpleFloorPlanner};
    use crate::halo2::dev::MockProver;
    use crate::halo2::plonk::{Circuit, ConstraintSystem, Error};
    use crate::main_gate::five::main_gate::MainGate;
    use crate::main_gate::five::NUMBER_OF_LOOKUP_LIMBS;
    use crate::{MainGateInstructions, UnassignedValue};

    cfg_if::cfg_if! {
        if #[cfg(feature = "kzg")] {
            use crate::halo2::pairing::bn256::Fr as Fp;
        } else {
            // default feature
            use crate::halo2::pasta::Fp;
        }
    }

    #[derive(Clone, Debug)]
    struct TestCircuitConfig {
        range_config: RangeConfig,
    }

    impl TestCircuitConfig {
        fn fine_tune_bit_lengths() -> Vec<usize> {
            (1..Self::base_bit_len()).collect()
        }

        fn base_bit_len() -> usize {
            16
        }

        fn new<F: FieldExt>(meta: &mut ConstraintSystem<F>) -> Self {
            let main_gate_config = MainGate::<F>::configure(meta);
            let fine_tune_bit_lengths = Self::fine_tune_bit_lengths();
            let range_config =
                RangeChip::<F>::configure(meta, &main_gate_config, fine_tune_bit_lengths);
            Self { range_config }
        }

        fn main_gate<F: FieldExt>(&self) -> MainGate<F> {
            MainGate::<F>::new(self.range_config.main_gate_config.clone())
        }

        fn range_chip<F: FieldExt>(&self) -> RangeChip<F> {
            RangeChip::<F>::new(self.range_config.clone(), Self::base_bit_len())
        }
    }

    #[derive(Default, Clone, Debug)]
    struct TestCircuit<F: FieldExt> {
        input: Vec<(usize, Option<F>)>,
    }

    impl<F: FieldExt> Circuit<F> for TestCircuit<F> {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            TestCircuitConfig::new(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let range_chip = config.range_chip();
            let main_gate = config.main_gate();

            layouter.assign_region(
                || "region 0",
                |mut region| {
                    let mut offset = 0;

                    for value in self.input.iter() {
                        let bit_len = value.0;
                        let value = value.1;

                        let a_0 = main_gate.assign_value(
                            &mut region,
                            &UnassignedValue(value),
                            &mut offset,
                        )?;

                        let a_1 = range_chip.range_value(
                            &mut region,
                            &UnassignedValue(value),
                            bit_len,
                            &mut offset,
                        )?;

                        main_gate.assert_equal(&mut region, a_0, a_1, &mut offset)?;
                    }

                    Ok(())
                },
            )?;

            #[cfg(not(feature = "no_lookup"))]
            range_chip.load_limb_range_table(&mut layouter)?;
            #[cfg(not(feature = "no_lookup"))]
            range_chip.load_overflow_range_tables(&mut layouter)?;

            Ok(())
        }
    }

    #[test]
    fn test_range_circuit() {
        let base_bit_len = TestCircuitConfig::base_bit_len();
        #[cfg(not(feature = "no_lookup"))]
        let k: u32 = (base_bit_len + 1) as u32;
        #[cfg(feature = "no_lookup")]
        let k: u32 = 8;

        let min_bit_len = 1;
        let max_bit_len = base_bit_len * (NUMBER_OF_LOOKUP_LIMBS + 1) - 1;

        let input = (min_bit_len..(max_bit_len + 1))
            .map(|i| {
                let bit_len = i as usize;
                let value = Some(Fp::from_u128((1 << i) - 1));
                (bit_len, value)
            })
            .collect();

        let circuit = TestCircuit::<Fp> { input };

        let prover = match MockProver::run(k, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));

        // // negative paths
        // for bit_len in min_bit_len..(max_bit_len + 1) {
        //     let input = vec![(bit_len, Some(Fp::from_u128(1 << bit_len)))];

        //     let circuit = TestCircuit::<Fp> { input };

        //     let prover = match MockProver::run(k, &circuit, vec![]) {
        //         Ok(prover) => prover,
        //         Err(e) => panic!("{:#?}", e),
        //     };
        //     assert_ne!(prover.verify(), Ok(()));
        // }
    }
}
