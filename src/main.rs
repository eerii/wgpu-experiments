fn main() {
    pollster::block_on(wgpu_test::run());
}
