# Simple Audio visualizer

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

Once that is done, go ahead and play some audio! You should see that the terminal interface is now active and the visualizer is working!
