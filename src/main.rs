// Let's write a simple circuit that proves knowledge of  x + 3 = 5, where x = 2.

use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::halo2curves::pasta::Fp;
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Instance, Selector};
use halo2_proofs::poly::Rotation;

#[derive(Clone, Debug, Default)]
pub struct TestCircuit<F: Field> {
    x: Value<F>,
    y: Value<F>,
}

#[derive(Clone, Debug)]
pub struct TestConfig {
    x: Column<Advice>,
    y: Column<Advice>,
    instance: Column<Instance>,
    selector: Selector,
}

impl TestConfig {
    pub fn configure<F: Field>(meta: &mut ConstraintSystem<F>) -> Self {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let instance = meta.instance_column();
        let selector = meta.selector();

        meta.enable_equality(x);
        meta.enable_equality(y);
        meta.enable_equality(instance);

        // Define our own gate!
        meta.create_gate("Test bear's first gate", |meta| {
            // x   | y   / out   /selector
            // x_0 / y_0 / out_0 / s_0
            // x_1 / y_1 / out_1 / s_1
            // x_2 / y_2 / out_2 / s_2
            // ...
            // with constraints:
            // s_i * (x_i + y_i - out_i ) = 0

            let advice_x = meta.query_advice(x, Rotation::cur());
            let advice_y = meta.query_advice(y, Rotation::cur());
            let out = meta.query_advice(x, Rotation::next());
            let s = meta.query_selector(selector);

            vec![s * (advice_x + advice_y - out)]
        });

        Self {
            x,
            y,
            instance,
            selector,
        }
    }
}

impl<F: Field> Circuit<F> for TestCircuit<F> {
    type Config = TestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        TestConfig::configure::<F>(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        let out = layouter.assign_region(
            || "Try synthesize",
            |mut region| {
                let x = region.assign_advice(|| "private input x", config.x, 0, || self.x)?;
                let y = region.assign_advice(|| "private input y", config.y, 0, || self.y)?;

                config.selector.enable(&mut region, 0)?;

                let value = x.value().copied() + y.value().copied();
                region.assign_advice(|| "out", config.x, 1, || value)
            },
        )?;
        layouter.constrain_instance(out.cell(), config.instance, 0)?;
        Ok(())
    }
}

fn main() {
    // TODO: Generate a universal setup key

    let x = Fp::from(2);
    let y = Fp::from(3);
    let c = Fp::from(5);
    let circuit = TestCircuit {
        x: Value::known(x),
        y: Value::known(y),
    };

    let prover = MockProver::run(4, &circuit, vec![vec![c]]).unwrap();
    let result = prover.verify();
    println!("{:?}", result);
}
