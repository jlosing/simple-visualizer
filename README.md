# Simple Terminal Audio visualizer

This is a simple audio visualizer I coded in rust. It is just a personal project for me to mess with audio signals, pipewire, and CPAL (cross platform audio library). Right now, it is only compatible on Linux devices using pipewire.

# Instructions

To run the file, download the v1.0.0 release executable file. Once it has downloaded, go to the directory where the file is and type:

```bash
chmod +x rust-visualizer
```

Then, to run the program, simply type:

```bash
./rust-visualizer
```

And the visualizer will start running. However it may not be on the correct input device. That is what the next section will solve

## Pavucontrol

If you are not familiar with pavucontrol, it is a graphical interface for PulseAudio. We are using Pipewire with PulseAudio so this works just fine. You can simply type `pavucontrol` in a terminal and it will open the interface.

You will want to open the `Recording` tab in pavucontrol. In there, you should see something like `ALSA plug-in [rust-visualizer]` Go ahead and click the drop down there, and you will want to select "Monitor of (YOUR SPEAKER HERE)".

Once that is done, go ahead and play some audio! You should see that the terminal interface is now active and the visualizer is working! Also note that changing the monitor affects it every time you run it, so you should be able to set the monitor and forget it. 

# What I used:

For this project, my language was Rust. This is because I wanted to familiarize myself more with the language. I use CPAL running with ALSA to capture audio and then I use the `pipewire` channel in ALSA. Sometime in the future I plan to make it more seamless to monitor audio, but right now you must manually change it in pavucontrol. I use ratatui for the TUI, and the barcharts aren't the best but they get the job done. The program uses `rustfft` for the fft calculations. This is what converts all the samples into my 16 bands of eq. 
