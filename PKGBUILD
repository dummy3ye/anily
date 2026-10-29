# Maintainer: <your-name>
pkgname=anily
pkgver=0.1.2
pkgrel=1
pkgdesc="A fast terminal-based anime streaming and discovery CLI"
arch=(x86_64 aarch64)
url="https://github.com/dummy3ye/anily"
license=("MIT" "Apache-2.0")
depends=(mpv)
makedepends=(cargo rust)
checkdepends=(cargo)
source=("$pkgname-$pkgver.tar.gz::https://github.com/dummy3ye/anily/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=("SKIP")

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 target/release/anily "$pkgdir/usr/bin/anily"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
