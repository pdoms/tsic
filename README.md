# tsic

A command line tool to configure metronome click tracks with changing time signatures (or not). Tap mode included, as well as small visual clues while playback runs. 

## Install

Download the latest binary for your platform from the [releases page](https://github.com/YOUR_USERNAME/tsic/releases/latest).

| Platform | Binary |
|----------|--------|
| Linux    | `tsic` |
| Windows  | `tsic.exe` |

Or, if you have the rust toochain [installed](https://rust-lang.org/tools/install/) fetch this repository and run:

```console
cargo build --release 
```

## Profiles

Profiles hold the following properties:

| key                       | type       | comment                                       |
|---------------------------|------------| ----------------------------------------------|
| name                      | `string`   | Name (identifier) of the project              |
| sample rate               | `integer`  | Sample rate - default: 44100                  |
| bpm                       | `integer`  | Beats per minute - default: 120               |
| beats_per_bar             | `integer`  | Beats per bar - default 4                     |
| beat unit                 | `integer`  | Beat unit (length of note) - default 4        |
| num_measures_fallback     | `integer`  | fallback section length                       |
| sound.freqency_hz         | `float`    | the frequency in hertz of click - def: 1400hz |
| sound.accent_frequency_hz | `float`    | the frequency in hertz of accent - def: 800hz |
| sound.sound_duration_secs | `float`    | length of click sound - default 0.2 secs      |
| sound.envelope_decay_secs | `float`    | when decay starts - default 0.1 seconds       |

Note: `profile.name` should be unique. Otherwise, an existing profile might
get overwritten. This might be fixed in a future version.

The following commands can be used in matters of profiles:

**defaults**
Prints the profile default values.

**profile**

```console
Create a Profile - for default values, run the 'defaults' command

Usage: tsic profile [OPTIONS] <NAME>

Arguments:
  <NAME>  The name of the profile. it will be stored as ./profiles/<NAME>.toml, If profile does not exist, and no optional arguments are provided, default arguments are stored. If it already exists, it will be updated. If profile does exist and no arguments are provided, the profile will be deleted

Options:
      --sr <SR>      Sample Rate
      --bpm <BPM>    Beats Per Minute
      --bpb <BPB>    Beats Per Bar
      --bu <BU>      Beats Unit
      --freq <FREQ>  Base Click Frequency in hz (decimal)
      --acc <ACC>    Accent Click Frequency in hz (decimal)
      --dur <DUR>    Duration in seconds (decimal) of the click
      --dec <DEC>    Duration of the envelope decay in seconds (decimal)
      --fb <FB>      A fallback for sections without number of measurements
```
Options are all optional if  not profided, values from the default profile 
will be used.

**profiles-list**
```console
Lists all available profile names

Usage: tsic profiles-list
```

## Projects

A project is a collection of `sections` that can have different time-signatures.

Base properties of a project are:

| key       | type     | comment                                    |
|-----------|----------|--------------------------------------------|
| name      | `string` | name of the project (main identifier)      |
| profile   | `string` | name of the `profile` this project uses    |
| file_path | `string` | where the project toml file is stored      |
| sections  | `list`   | the sections of the project                |

Note: `project.name` should be unique. Otherwise, an existing profile might
get overwritten. This might be fixed in a future version.

The properites of a section:
| key             | type      | comment                                    |
|-----------------|-----------|--------------------------------------------|
| bpm             | `integer` | beats per minute of this section           |
| measures        | `integer` | how many measures this section has         |
| name            | `string`  | the name of the section (optional)         |
| time_signature  | `string`  | the time signature of the unit - provides as `4/4` or `7/8` where `7` represents the `beats_per_bar` and `8` the `beat_unit`         |

The following commands can be used in matters of projects:

**projects-list**
```console
Lists all available project names

Usage: tsic projects-list
```

**project**
```console
Prints the project of the provided name to stdout. Note: the profile is not printed

Usage: tsic project <NAME>

Arguments:
  <NAME>
````

**new**
```console
Creates a new project

Usage: tsic new [OPTIONS] <NAME>

Arguments:
  <NAME>  

Options:
  -p, --profile <PROFILE>  [default: default]
```

**append** _a section__
```console
Append a section to the current project. Except for <measures>, omitted values are taken from the previous section. If there is  no previous section, it defaults to profile values

Usage: tsic append [OPTIONS] <PROJECT_NAME>

Arguments:
  <PROJECT_NAME>  name of the profile

Options:
  -s, --section-name <SECTION_NAME>  name of the section
  -b, --bpm <BPM>                    Beats Per Minute
  -t, --time-sig <TIME_SIG>          Time Signature
  -m, --measures <MEASURES>          Number of Measures this section has If the section before the new section is open ended, this will throw an error
```

**insert** _a section_
```console
Insert a section at the provided index. Omitted values are taken from the previous section. If there is  no previous section it defaults to profile defaults

Usage: tsic insert [OPTIONS] --position <POSITION> --measures <MEASURES> <PROJECT_NAME>

Arguments:
  <PROJECT_NAME>  name of the project to load

Options:
  -p, --position <POSITION>          the index at which the section should be inserted
  -b, --bpm <BPM>                    Beats Per Minute
  -t, --time-sig <TIME_SIG>          Time Signature
  -m, --measures <MEASURES>          Number of Measures this section has
  -s, --section-name <SECTION_NAME>  A name for the section
```
**edit** _a section_

```console
Edit a section

Usage: tsic edit [OPTIONS] <PROJECT_NAME>

Arguments:
  <PROJECT_NAME>  name of the project

Options:
  -p, --position <POSITION>          the index/position of the section to be edited (either name or position is required)
  -n, --name <NAME>                  name of the section (either name or position is required)
  -b, --bpm <BPM>                    Beats Per Minute
  -t, --time-sig <TIME_SIG>          Time Signature
  -m, --measures <MEASURES>          Number of Measures this section has
      --section-name <SECTION_NAME>  A new name for the section
```
**remove-section**

```console
Remove a section

Usage: tsic remove-section [OPTIONS] <PROJECT_NAME>

Arguments:
  <PROJECT_NAME>  the name of the project

Options:
  -p, --position <POSITION>          the index/position of the section to be deleted (either name or position is required)
  -s, --section-name <SECTION_NAME>  name of the section (either name or position is required)
```

## Fileformat Commands 

These commands open a project and render it to a specified format to disk.

**wav**

```console
Usage: tsic wav <PROJECT_NAME> [OUTFILE]

Arguments:
  <PROJECT_NAME>  the name of the project
  [OUTFILE]       provide path or name to outfile. Defaults to "./Wav.name.wav"
```

**midi**

```console
Writes the project to disk in the midi format. Midi defaults can be printed but not used in a profile, so values are provided here

Usage: tsic midi [OPTIONS] <PROJECT_NAME> [OUTFILE]

Arguments:
  <PROJECT_NAME>  name of the project
  [OUTFILE]       provide path or name to outfile. Defaults to "./Wav.name.wav"

Options:
  -c, --channel <CHANNEL>                midi channel (0-15 - zero-indexed)
  -t, --ticks-per-beat <TICKS_PER_BEAT>  ticks per beat (quarter note)
  -a, --accent <ACCENT>                  accent note
  -n, --normal <NORMAL>                  normal (click) note
  -d, --duration <DURATION>              how the note is held
      --vel-accent <VEL_ACCENT>          velocity accent
      --vel-normal <VEL_NORMAL>          velocity normal
  -h, --help                             Print help
```

## Interactive Commands

These commands are interactive or play sounds.

**play**

```console
Plays the provided track/project

Usage: tsic play [OPTIONS] <PROJECT_NAME>

Arguments:
  <PROJECT_NAME>  

Options:
  -v, --visualize  whether to run visualization while playing
```

**tap**

```console
Tab with the space bar a temp and display it in the terminal

Usage: tsic tap
```

**metronome**

```console
Provide bpm, signature and a config and have the mentronome ticking away

Usage: tsic metronome [OPTIONS]

Options:
  -b, --bpm <BPM>              the desired bpm
  -s, --signature <SIGNATURE>  time signature (e.g. 4/4) can be provided or taken from the a profile
  -p, --profile <PROFILE>      provide a profile as a template. If not provided, it falls back to the default profile
```

## License

This program is licensed under the MIT license.
