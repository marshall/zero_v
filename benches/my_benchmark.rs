use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zero_v::{compose, compose_nodes, Composite, NestLevel, NextNode, Node};

trait IntOp {
    fn execute(&self, input: usize) -> usize;
}

trait IntOpAtLevel {
    fn execute_at_level(&self, input: usize, level: usize) -> Option<usize>;
}

trait IterIntOps<NodeType: NextNode + IntOpAtLevel> {
    fn iter_execute(&self, input: usize) -> CompositeIterator<'_, NodeType>;
}

impl IntOpAtLevel for () {
    fn execute_at_level(&self, _input: usize, _level: usize) -> Option<usize> {
        None
    }
}

impl<A: IntOp, B: NextNode + IntOpAtLevel + NestLevel> IntOpAtLevel for Node<A, B> {
    fn execute_at_level(&self, input: usize, level: usize) -> Option<usize> {
        if level == self.nest_level() {
            Some(self.data.execute(input))
        } else {
            self.next.execute_at_level(input, level)
        }
    }
}

impl<Nodes: NextNode + IntOpAtLevel + NestLevel> IterIntOps<Nodes> for Composite<Nodes> {
    fn iter_execute(&self, input: usize) -> CompositeIterator<'_, Nodes> {
        CompositeIterator::new(&self.head, input, self.head.nest_level())
    }
}

struct CompositeIterator<'a, Nodes: NextNode + IntOpAtLevel> {
    level: usize,
    input: usize,
    parent: &'a Nodes,
}

impl<'a, Nodes: NextNode + IntOpAtLevel> CompositeIterator<'a, Nodes> {
    fn new(parent: &'a Nodes, input: usize, max_level: usize) -> Self {
        Self {
            parent,
            input,
            level: max_level,
        }
    }
}

impl<'a, Nodes: NextNode + IntOpAtLevel> Iterator for CompositeIterator<'a, Nodes> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.parent.execute_at_level(self.input, self.level);
        if self.level > 0 {
            self.level -= 1
        };
        result
    }
}

struct Adder {
    value: usize,
}

impl Adder {
    fn new(value: usize) -> Self {
        Self { value }
    }
}

impl IntOp for Adder {
    fn execute(&self, input: usize) -> usize {
        input + self.value
    }
}

struct Multiplier {
    value: usize,
}

impl Multiplier {
    fn new(value: usize) -> Self {
        Self { value }
    }
}

impl IntOp for Multiplier {
    fn execute(&self, input: usize) -> usize {
        input * self.value
    }
}

struct RShifter {
    value: usize,
}

impl RShifter {
    fn new(value: usize) -> Self {
        Self { value }
    }
}

impl IntOp for RShifter {
    fn execute(&self, input: usize) -> usize {
        input >> self.value
    }
}

struct LShifter {
    value: usize,
}

impl LShifter {
    fn new(value: usize) -> Self {
        Self { value }
    }
}

impl IntOp for LShifter {
    fn execute(&self, input: usize) -> usize {
        input << self.value
    }
}

struct ConstAdder<const VALUE: usize> {}

impl<const VALUE: usize> ConstAdder<VALUE> {
    fn new() -> Self {
        Self {}
    }
}

impl<const VALUE: usize> IntOp for ConstAdder<VALUE> {
    fn execute(&self, input: usize) -> usize {
        input + VALUE
    }
}

struct ConstMultiplier<const VALUE: usize> {}

impl<const VALUE: usize> ConstMultiplier<VALUE> {
    fn new() -> Self {
        Self {}
    }
}

impl<const VALUE: usize> IntOp for ConstMultiplier<VALUE> {
    fn execute(&self, input: usize) -> usize {
        input * VALUE
    }
}

struct ConstRShifter<const VALUE: usize> {}

impl<const VALUE: usize> ConstRShifter<VALUE> {
    fn new() -> Self {
        Self {}
    }
}

impl<const VALUE: usize> IntOp for ConstRShifter<VALUE> {
    fn execute(&self, input: usize) -> usize {
        input >> VALUE
    }
}

struct ConstLShifter<const VALUE: usize> {}

impl<const VALUE: usize> ConstLShifter<VALUE> {
    fn new() -> Self {
        Self {}
    }
}

impl<const VALUE: usize> IntOp for ConstLShifter<VALUE> {
    fn execute(&self, input: usize) -> usize {
        input >> VALUE
    }
}

fn bench_composed<NodeType, Composed>(input: usize, composed: &Composed) -> usize
where
    NodeType: IntOpAtLevel + NextNode,
    Composed: IterIntOps<NodeType>,
{
    composed.iter_execute(input).sum()
}

fn bench_trait_objects(input: usize, ops: &Vec<Box<dyn IntOp>>) -> usize {
    ops.iter().map(|op| op.execute(input)).sum()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Adders");

    let ops_dyn: Vec<Box<dyn IntOp>> = vec![
        Box::new(Adder::new(0)),
        Box::new(LShifter::new(1)),
        Box::new(Adder::new(2)),
        Box::new(Multiplier::new(3)),
        Box::new(Adder::new(4)),
        Box::new(Multiplier::new(5)),
        Box::new(Adder::new(6)),
        Box::new(Multiplier::new(7)),
        Box::new(Adder::new(8)),
        Box::new(Multiplier::new(9)),
        Box::new(Adder::new(10)),
        Box::new(RShifter::new(11)),
        Box::new(Adder::new(12)),
        Box::new(RShifter::new(13)),
    ];

    let ops_dyn_const: Vec<Box<dyn IntOp>> = vec![
        Box::new(ConstAdder::<0>::new()),
        Box::new(ConstLShifter::<1>::new()),
        Box::new(ConstAdder::<2>::new()),
        Box::new(ConstMultiplier::<3>::new()),
        Box::new(ConstAdder::<4>::new()),
        Box::new(ConstMultiplier::<5>::new()),
        Box::new(ConstAdder::<6>::new()),
        Box::new(ConstMultiplier::<7>::new()),
        Box::new(ConstAdder::<8>::new()),
        Box::new(ConstMultiplier::<9>::new()),
        Box::new(ConstAdder::<10>::new()),
        Box::new(ConstRShifter::<11>::new()),
        Box::new(ConstAdder::<12>::new()),
        Box::new(ConstRShifter::<13>::new()),
    ];

    let ops = compose!(
        Adder::new(0),
        LShifter::new(1),
        Adder::new(2),
        Multiplier::new(3),
        Adder::new(4),
        Multiplier::new(5),
        Adder::new(6),
        Multiplier::new(7),
        Adder::new(8),
        Multiplier::new(9),
        Adder::new(10),
        RShifter::new(11),
        Adder::new(12),
        RShifter::new(13)
    );

    let ops_const = compose!(
        ConstAdder::<0>::new(),
        ConstLShifter::<1>::new(),
        ConstAdder::<2>::new(),
        ConstMultiplier::<3>::new(),
        ConstAdder::<4>::new(),
        ConstMultiplier::<5>::new(),
        ConstAdder::<6>::new(),
        ConstMultiplier::<7>::new(),
        ConstAdder::<8>::new(),
        ConstMultiplier::<9>::new(),
        ConstAdder::<10>::new(),
        ConstRShifter::<11>::new(),
        ConstAdder::<12>::new(),
        ConstRShifter::<13>::new()
    );

    group.bench_function("static dispatch/arg", |b| {
        b.iter(|| bench_composed(black_box(20), black_box(&ops)))
    });
    group.bench_function("dynamic dispatch/arg", |b| {
        b.iter(|| bench_trait_objects(black_box(20), black_box(&ops_dyn)))
    });
    group.bench_function("static dispatch/const", |b| {
        b.iter(|| bench_composed(black_box(20), black_box(&ops_const)))
    });
    group.bench_function("dynamic dispatch/const", |b| {
        b.iter(|| bench_trait_objects(black_box(20), black_box(&ops_dyn_const)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);