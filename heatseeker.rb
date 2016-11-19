class Heatseeker < Formula
  desc "A high-performance general purpose fuzzy finder, based on selecta"
  homepage "https://github.com/rschmitt/heatseeker"
  url "https://github.com/rschmitt/heatseeker/releases/download/v1.5.0/hs-mac"
  sha256 "e23bca0931e16f01fad78e967692487b058f6c51e86a769acf1cd8e4a91f9de3"

  def install
    system *%W(mv hs-mac hs)
    bin.install "hs"
  end

  test do
    system *%W(#{bin}/hs -v)
    assert_equal "cc\n", `echo 'aa\\nbb\\ncc\\n' | #{bin}/hs -s c -f`
  end
end
