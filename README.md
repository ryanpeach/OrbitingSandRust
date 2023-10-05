# OrbitingSandRust

# Rust

## Reasons for Moving to Rust

1. Multiprocessing - We need a A LOT of parallelism to make this work. Rust is a great language for this.
2. ECS - Without an engine, we need an ecs to impose some structure on our code. Rust has great ECS libraries.
3. No Cmake - Need I say more?

## Challenges

1. We are not going to use Bevy because it is so alpha. Many of our libraries are alpha.
2. Bindings - We need to bind to liquidfun, which will be a pain.