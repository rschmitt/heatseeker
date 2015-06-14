class Heatseeker < Formula
  desc "A high-performance general purpose fuzzy finder, based on selecta"
  homepage "https://github.com/rschmitt/heatseeker"
  url "https://github.com/rschmitt/heatseeker/archive/v1.1.0.tar.gz"
  sha256 "2797fba1ee5a1beadb97ad4b7a7847964a0b38517af049deeb9e873a07be4417"

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
