class Heatseeker < Formula
  desc "A high-performance general purpose fuzzy finder, based on selecta"
  homepage "https://github.com/rschmitt/heatseeker"
  url "https://github.com/rschmitt/heatseeker/archive/v1.0.1.tar.gz"
  sha256 "44bb16a6a650063a7268b08fa406d73fc7a234e1c674e54ef6c9a02c1f159499"

  depends_on "rust"

  def install
    system *%w(cargo build --release)
    bin.install "target/release/hs"
  end

  test do
    system *%W(#{bin}/hs -v)
    assert_equal "cc\n", `echo 'aa\\nbb\\ncc\\n' | #{bin}/hs -s c -f`
  end
end
