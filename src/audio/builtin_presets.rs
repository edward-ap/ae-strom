use crate::model::EqPreset;
use once_cell::sync::Lazy;

/// A constant array representing the standard center frequencies (in Hertz)
/// typically used for a 10-band equalizer.
///
/// These frequencies represent the center points of the bands that can be
/// adjusted in an audio equalizer, with each value corresponding to a specific
/// part of the audio spectrum:
///
/// - 32 Hz: Sub-bass
/// - 64 Hz: Bass
/// - 125 Hz: Lower midrange
/// - 250 Hz: Midrange
/// - 500 Hz: Upper midrange
/// - 1000 Hz: Lower treble
/// - 2000 Hz: Mid treble
/// - 4000 Hz: Upper treble
/// - 8000 Hz: Presence
/// - 16000 Hz: Brilliance
///
/// This array can be used in audio processing software to configure or display
/// equalization bands.
///
/// # Example
///
/// ```
/// use ae_strom::presets::EQ_FREQUENCIES;
///
/// for frequency in EQ_FREQUENCIES {
///     println!("EQ Band Frequency: {} Hz", frequency);
/// }
/// ```
pub const EQ_FREQUENCIES: [u32; 10] = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

/// A static, lazily initialized collection of built-in equalizer (EQ) presets.
///
/// This structure uses the `Lazy` type from the `once_cell` crate to initialize
/// the presets only once when accessed for the first time.
///
/// Each `EqPreset` represents a named equalizer configuration consisting of:
/// - A `name`, which is a descriptive string indicating the preset type.
/// - A `preamp` value, which specifies the overall gain adjustment (default is 0.0).
/// - A `bands` array containing 10 floating-point values representing the adjustment
///   levels for specific frequency bands. The values in the `bands` array correspond
///   to the gain (boost/cut) at various frequencies, typically arranged from low
///   frequencies to high frequencies.
///
/// The following built-in EQ presets are included, along with their respective band
/// adjustments:
///
/// - "Off": Flat EQ with no adjustments.
/// - "Acoustic": Boosts mids and highs, whereas lows remain untouched.
/// - "Classic": Slight attenuation of higher frequencies.
/// - "Club": Enhances some mid frequency ranges.
/// - "Dance": Strong bass boost and a slight cut in mid frequencies.
/// - "Deep": Boosts low and lowers mid/high frequencies for a deep sound.
/// - "Electronic": Mixed frequency adjustments suited for electronic music.
/// - "Flat": Flat equalizer with no band enhancement or attenuation.
/// - "Full Bass": Enhances lower frequencies with minimal effect on others.
/// - "Full Bass & Treble": Boosts both low and high frequencies.
/// - "Full Treble": Enhances high frequencies with minimal effect on others.
/// - "Full Vocal": Adjusted for clearer vocals by reducing bass and boosting mids.
/// - "Hip-Hop": Strong bass boost and minor changes to mid-frequency ranges.
/// - "Jazz": Subtle adjustments for warm sound ideal for jazz music.
/// - "Latin": Slight boost at low and high frequencies while cutting mids.
/// - "Live": Boosted mids and high frequencies for a live performance feel.
/// - "Loudness": Strong emphasis on bass and high frequencies.
/// - "Lounge": Soft adjustments emphasizing bass and lower mids.
/// - "Metal": Enhanced mids and highs for a sharp sound.
/// - "Piano": Boosts mids and highs for clearer piano sound.
/// - "Pop": Adjustments for clarity and brightness in pop music.
/// - "Reggae": Subtle equivalences with a slight mid-cut for reggae music.
/// - "RnB": Boosted clarity with emphasis on mids and highs for RnB music.
/// - "Rock": Aggressive enhancements on highs and lows for rock music.
/// - "Ska": Boosts highs and mids while cutting low frequencies.
/// - "Soft Rock": Gentle enhancements with slight adjustments across bands.
/// - "Soft": Balanced adjustments with emphasis on vocals and melody.
/// - "Techno": Boosted mids and high frequencies for a punchy techno sound.
/// - "Voice": Tailored for vocal clarity by enhancing mids and highs.
///
/// Example usage:
/// ```rust
/// use ae_strom::presets::BUILTIN_EQ_PRESETS;
/// // Access the built-in EQ presets:
/// let eq_presets = &*BUILTIN_EQ_PRESETS;
///
/// // Iterate through the presets:
/// for preset in eq_presets {
///     println!("Preset: {}", preset.name);
/// }
/// ```
///
/// Use these presets as default starting points for equalizer adjustments
/// tailored to common music genres or audio preferences.
pub static BUILTIN_EQ_PRESETS: Lazy<Vec<EqPreset>> = Lazy::new(|| {
    vec![
        EqPreset {
            name: "Off".to_string(),
            preamp: 0.0,
            bands: [0.0; 10],
        },
        EqPreset {
            name: "Acoustic".to_string(),
            preamp: 0.0,
            bands: [4.8, 4.5, 3.5, 0.4, 1.8, 1.6, 3.3, 3.8, 3.3, 1.3],
        },
        EqPreset {
            name: "Classic".to_string(),
            preamp: 0.0,
            bands: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.5, -3.7, -5.3],
        },
        EqPreset {
            name: "Club".to_string(),
            preamp: 0.0,
            bands: [0.0, 0.0, 2.3, 3.5, 3.5, 3.5, 2.6, 0.0, 0.0, 0.0],
        },
        EqPreset {
            name: "Dance".to_string(),
            preamp: 0.0,
            bands: [6.8, 5.3, 4.0, 2.0, 0.0, 0.0, 0.0, -3.1, -3.1, 0.0],
        },
        EqPreset {
            name: "Deep".to_string(),
            preamp: 0.0,
            bands: [4.5, 3.0, 1.3, 0.4, 2.3, 1.8, 1.1, -2.8, -4.0, -5.0],
        },
        EqPreset {
            name: "Electronic".to_string(),
            preamp: 0.0,
            bands: [4.0, 3.5, 0.9, -0.6, -2.6, 1.8, 0.4, 0.9, 3.5, 4.3],
        },
        EqPreset {
            name: "Flat".to_string(),
            preamp: 0.0,
            bands: [0.0; 10],
        },
        EqPreset {
            name: "Full Bass".to_string(),
            preamp: 0.0,
            bands: [5.0, 4.0, 3.0, 2.1, 1.1, 0.4, 0.4, 0.4, 0.4, 0.4],
        },
        EqPreset {
            name: "Full Bass & Treble".to_string(),
            preamp: 0.0,
            bands: [5.0, 4.0, 3.0, 2.1, 1.1, 0.6, 2.1, 3.3, 3.8, 5.2],
        },
        EqPreset {
            name: "Full Treble".to_string(),
            preamp: 0.0,
            bands: [0.4, 0.4, 0.4, 0.4, 0.4, 0.6, 2.1, 3.3, 3.8, 5.2],
        },
        EqPreset {
            name: "Full Vocal".to_string(),
            preamp: 0.0,
            bands: [-2.1, -3.3, -3.3, 0.9, 3.3, 3.3, 2.6, 1.0, -0.3, -2.1],
        },
        EqPreset {
            name: "Hip-Hop".to_string(),
            preamp: 0.0,
            bands: [4.5, 3.8, 0.9, 2.3, -1.3, -1.3, 1.1, -1.3, 1.8, 2.3],
        },
        EqPreset {
            name: "Jazz".to_string(),
            preamp: 0.0,
            bands: [3.8, 2.3, 0.9, 1.8, -1.8, -1.8, -0.6, 1.1, 2.6, 3.5],
        },
        EqPreset {
            name: "Latin".to_string(),
            preamp: 0.0,
            bands: [4.0, 2.3, -0.6, -0.6, -1.8, -1.8, -1.8, -0.4, 2.6, 4.0],
        },
        EqPreset {
            name: "Live".to_string(),
            preamp: 0.0,
            bands: [-4.6, -2.9, -0.4, 1.8, 3.1, 3.7, 3.7, 2.6, 1.3, 0.4],
        },
        EqPreset {
            name: "Loudness".to_string(),
            preamp: 0.0,
            bands: [5.2, 3.5, -0.6, -0.6, -2.6, -0.4, -1.6, -5.5, 4.5, 0.4],
        },
        EqPreset {
            name: "Lounge".to_string(),
            preamp: 0.0,
            bands: [-3.5, -1.8, -1.1, 0.9, 3.5, 1.8, -0.4, -1.8, 1.8, 0.4],
        },
        EqPreset {
            name: "Metal".to_string(),
            preamp: 0.0,
            bands: [-0.3, 2.9, 2.7, -0.7, -2.7, -3.1, 0.0, 3.1, 5.7, 2.9],
        },
        EqPreset {
            name: "Piano".to_string(),
            preamp: 0.0,
            bands: [2.6, 1.6, -0.6, 2.1, 2.6, 0.6, 3.3, 4.0, 2.3, 3.0],
        },
        EqPreset {
            name: "Pop".to_string(),
            preamp: 0.0,
            bands: [-2.4, -0.9, 1.8, 3.5, 4.6, 3.3, 1.5, 0.0, -0.9, -1.1],
        },
        EqPreset {
            name: "Reggae".to_string(),
            preamp: 0.0,
            bands: [0.0, 0.0, 0.0, -1.3, -3.7, -0.7, 2.0, 3.3, 1.8, 0.0],
        },
        EqPreset {
            name: "RnB".to_string(),
            preamp: 0.0,
            bands: [2.1, 6.2, 5.5, 1.1, -2.8, -1.8, 2.1, 2.6, 2.3, 3.5],
        },
        EqPreset {
            name: "Rock".to_string(),
            preamp: 0.0,
            bands: [5.9, 4.8, 1.5, -1.8, -4.6, -1.1, 2.6, 5.5, 6.6, 7.0],
        },
        EqPreset {
            name: "Ska".to_string(),
            preamp: 0.0,
            bands: [-1.8, -3.0, -2.8, -0.4, 2.6, 3.5, 5.5, 6.0, 6.5, 6.0],
        },
        EqPreset {
            name: "Soft Rock".to_string(),
            preamp: 0.0,
            bands: [2.6, 2.6, 1.3, -0.4, -2.8, -3.5, -2.6, -0.4, 1.8, 2.6],
        },
        EqPreset {
            name: "Soft".to_string(),
            preamp: 0.0,
            bands: [3.0, 0.9, -0.9, -1.8, -1.1, 2.8, 5.2, 6.0, 6.5, 7.2],
        },
        EqPreset {
            name: "Techno".to_string(),
            preamp: 0.0,
            bands: [4.8, 3.5, 0.1, -3.5, -3.0, 0.1, 4.8, 6.0, 6.0, 5.7],
        },
        EqPreset {
            name: "Voice".to_string(),
            preamp: 0.0,
            bands: [-4.3, -1.1, -0.4, 0.1, 3.0, 4.3, 4.8, 3.8, 1.8, -0.6],
        },
    ]
});

/// Returns a list of names of all built-in EQ (Equalizer) presets.
///
/// This function iterates over the list of built-in EQ presets (`BUILTIN_EQ_PRESETS`)
/// and collects their names into a `Vec<String>`. Each preset name is cloned
/// from the `BUILTIN_EQ_PRESETS` data structure to create the resulting vector.
///
/// # Returns
///
/// A `Vec<String>` containing the names of all built-in EQ presets.
///
/// # Example
///
/// ```
/// use ae_strom::presets::builtin_eq_preset_names;
/// let preset_names = builtin_eq_preset_names();
/// println!("{:?}", preset_names); // Outputs a list of EQ preset names
/// ```
///
/// Note: `BUILTIN_EQ_PRESETS` is assumed to be a global constant or static
/// variable that contains the preset information with a `name` field.
pub fn builtin_eq_preset_names() -> Vec<String> {
    BUILTIN_EQ_PRESETS.iter().map(|p| p.name.clone()).collect()
}

/// Finds a built-in equalizer preset by its name.
///
/// This function searches for a preset within the predefined collection of
/// built-in equalizer presets (`BUILTIN_EQ_PRESETS`) by comparing the given
/// `name` with the `name` field of each preset. If a matching preset is found,
/// it returns a reference to the preset; otherwise, it returns `None`.
///
/// # Parameters
/// - `name`: A string slice that holds the name of the equalizer preset to look for.
///
/// # Returns
/// - `Option<&'static EqPreset>`:
///     - `Some(&EqPreset)`: A reference to the matching preset if found.
///     - `None`: If no preset with the given name exists.
///
/// # Examples
/// ```
/// use ae_strom::presets::find_builtin_eq_preset;
/// let preset_name = "Rock";
/// if let Some(preset) = find_builtin_eq_preset(preset_name) {
///     println!("Preset found: {}", preset.name);
/// } else {
///     println!("Preset not found");
/// }
/// ```
///
/// # Notes
/// - The function assumes that `BUILTIN_EQ_PRESETS` is a collection of predefined presets
///   and that each preset has a `name` field of type `&str`.
/// - The lifetime of the returned reference is `'static` because the presets are statically defined.
pub fn find_builtin_eq_preset(name: &str) -> Option<&'static EqPreset> {
    BUILTIN_EQ_PRESETS.iter().find(|p| p.name == name)
}

/// Determines if a given name corresponds to a built-in EQ preset.
///
/// This function checks if the provided `name` matches any predefined
/// built-in equalizer (EQ) preset by leveraging the `find_builtin_eq_preset`
/// function. If a match is found, it returns `true`, otherwise `false`.
///
/// # Arguments
///
/// * `name` - A string slice representing the name of the preset to check.
///
/// # Returns
///
/// * `true` if the `name` matches a built-in EQ preset.
/// * `false` if no match is found.
///
/// # Example
///
/// ```rust
/// use ae_strom::presets::is_builtin_eq_preset;
/// let preset_name = "Rock";
/// let is_builtin = is_builtin_eq_preset(preset_name);
/// assert!(is_builtin);
/// ```
pub fn is_builtin_eq_preset(name: &str) -> bool {
    find_builtin_eq_preset(name).is_some()
}
