# GAT - GlazeWM Alternating Tiler, written in Rust.

### Why?
Really, there was no reason why other than exploring async and websockets using Rust. I haven't been messing around with programming
too much lately, so I figured that this would be a good starting point. Yes, it is shoddily-written and unnecessary, but why not?
And yes, this is based on the same premise as [Burgr033's work using Python, which can be found here.](https://github.com/burgr033/GlazeWM-autotiling-python)

-----
### What about-?
Yes, I know, you could use pytools to compile [the original script by Burgr033](https://github.com/burgr033/GlazeWM-autotiling-python) and run it as an executable, but I personally hate that approach and think that the packaging for Rust is superior in this regard.

Yes, I know that there's rookie mistakes and a patchy appearance, such as how it looks like things were slammed together in a ghetto Large Hadron Collider, and that's because they were. I shamelessly used the general outline from [jonhoo's OBS-DO project](https://github.com/jonhoo/obs-do) to setup async, then tossed on what I could find out there which happened to be a library called [Tungstenite](https://crates.io/crates/tungstenite).

-----
### Okay, so how would I set this up to build from scratch?
Well, download the source code and make sure that Rust is installed on your system. I tend to use the Nightly toolchain and the GNU GCC branch, which you can download and set as your default with the following:
```Rust
rustup install nightly-gnu
rustup default nightly-gnu
```

Open the project folder in a terminal,
then type `cargo b -r` to build a release binary, which will be located in `gat-gwm\target\release\gat-gwm.exe`.

If all of that was too much, download the pre-made binary.

From here, you can add the executable to a script, bind it to a key in GWM, or just have it autostart.

-----
### Wishlist:
- [X] Make a way to minimize this to tray or hide the window without crazy code expansion.
- [ ] Figure out why I'm so bad at programming.
- [ ] Do better.

-----
Hey, you can help or contribute if you want to. Just file a PR.
My own patchy work is GPLv3, the associated libraries are under their own licenses, assuming it even matters.
