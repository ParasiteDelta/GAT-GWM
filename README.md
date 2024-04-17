# GlaAlt - GlazeWM Alternating Autotiler, written in Rust.

### Why?
Really, there was no reason why other than exploring async and websockets using Rust. I haven't been messing around with programming 
too much lately, so I figured that this would be a good starting point. Yes, it is shoddily-written and unnecessary, but why not? 
And yes, this is based on the same premise as [Burgr033's work using Python, which can be found here.](https://github.com/burgr033/GlazeWM-autotiling-python)

### What about-?
Yes, I know, you could use pytools to compile [the original script by Burgr033](https://github.com/burgr033/GlazeWM-autotiling-python) and run it as an executable, but I personally hate that approach and think that the packaging for Rust is superior in this regard. 

Yes, I know that there's rookie mistakes and a patchy appearance, such as the unreachable code or how it looks like things were slammed together in a ghetto Large Hadron Collider, and that's because they were. I shamelessly used the general outline from [jonhoo's OBS-DO project](https://github.com/jonhoo/obs-do) to setup async, then tossed on what I could find out there which happened to be a library called [Tungstenite](https://crates.io/crates/tungstenite).

Hey, if you want to waste time improving it, go for it. Submit a PR, let's finalize this, or at least make it a bit better.
My own patchy work is GPLv3, the associated libraries are under their own licenses, assuming it even matters.