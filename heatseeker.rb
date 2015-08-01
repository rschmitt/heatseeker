class Heatseeker < Formula
  desc "A high-performance general purpose fuzzy finder, based on selecta"
  homepage "https://github.com/rschmitt/heatseeker"
  url "https://github.com/rschmitt/heatseeker/archive/v1.3.0.tar.gz"
  sha256 "9805d7e2e6542bcb157c7b2f86e66a8aaac97b204c99d54d62955e3519d4cd4d"

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
