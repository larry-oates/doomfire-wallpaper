# Maintainer: Larry Oates <leafman12345@gmail.com>
pkgname=doomfire-wallpaper
pkgver=1.0.0
pkgrel=1
pkgdesc="DOOM-style animated fire wallpaper for Hyprpaper"
arch=('x86_64')
giturl="https://github.com/Leafmun-certii/doom_fire_wallpaper"
license=('WTFPL')
depends=('hyprpaper' 'grim' 'rust' 'cargo')
makedepends=('git' 'cargo')
source=("$pkgname::git+$giturl")
md5sums=('SKIP')

build() {
  cd "$srcdir/$pkgname"
  cargo build --release --locked --bin doom-fire-wallpaper
}

package() {
  cd "$srcdir/$pkgname"
  install -Dm755 "target/release/doom-fire-wallpaper" "$pkgdir/usr/bin/doom-fire-wallpaper"
  install -Dm755 "dfpaper" "$pkgdir/usr/bin/dfpaper"
  install -Dm644 "README.MD" "$pkgdir/usr/share/doc/doomfire-wallpaper/README.MD"
}
