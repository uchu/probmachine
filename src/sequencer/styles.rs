use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum StylePattern {
    #[default]
    None,
    Classical,
    Blues,
    Jazz,
    Rock,
    Latin,
    Techno,
    Ambient,
    Reggae,
    Dubstep,
    Funk,
    MiddleEastern,
    Celtic,
}

impl StylePattern {
    pub fn all() -> &'static [StylePattern] {
        &[
            StylePattern::None,
            StylePattern::Classical,
            StylePattern::Blues,
            StylePattern::Jazz,
            StylePattern::Rock,
            StylePattern::Latin,
            StylePattern::Techno,
            StylePattern::Ambient,
            StylePattern::Reggae,
            StylePattern::Dubstep,
            StylePattern::Funk,
            StylePattern::MiddleEastern,
            StylePattern::Celtic,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            StylePattern::None => "None",
            StylePattern::Classical => "Classical",
            StylePattern::Blues => "Blues",
            StylePattern::Jazz => "Jazz",
            StylePattern::Rock => "Rock",
            StylePattern::Latin => "Latin",
            StylePattern::Techno => "Techno",
            StylePattern::Ambient => "Ambient",
            StylePattern::Reggae => "Reggae",
            StylePattern::Dubstep => "Dubstep",
            StylePattern::Funk => "Funk",
            StylePattern::MiddleEastern => "Middle Eastern",
            StylePattern::Celtic => "Celtic",
        }
    }

    /// Returns patterns sorted from simple (index 0) to complex (index 19).
    /// Each pattern is a list of interval directions/steps applied to enabled notes.
    /// Positive = steps up through enabled notes, negative = steps down, 0 = same note.
    pub fn patterns(&self) -> &'static [&'static [i8]] {
        match self {
            StylePattern::None => &[],

            // Classical: triads, scales, arpeggios, Alberti bass, sequences
            // Sorted: simple triads → scalar runs → complex inversions/sequences
            StylePattern::Classical => &[
                // 1. Root-root (unison repeat)
                &[0, 0],
                // 2. Simple ascending triad (root, 3rd, 5th)
                &[0, 1, 2],
                // 3. Descending triad
                &[0, -1, -2],
                // 4. Root-fifth oscillation
                &[0, 4, 0, 4],
                // 5. Ascending 4-note scale
                &[0, 1, 2, 3],
                // 6. Descending 4 notes
                &[0, -1, -2, -3],
                // 7. Alberti bass (root, 5th, 3rd, 5th)
                &[0, 4, 2, 4],
                // 8. Ascending scale fragment
                &[0, 1, 2, 3, 4],
                // 9. Descending scale fragment
                &[0, -1, -2, -3, -4],
                // 10. Pendulum (up-down-up)
                &[0, 1, 2, 1, 0],
                // 11. Broken chord up-down
                &[0, 2, 1, 3, 2],
                // 12. Mordent figure
                &[0, 1, 0, -1, 0],
                // 13. Scalar run 6 notes
                &[0, 1, 2, 3, 4, 5],
                // 14. Alberti extended
                &[0, 4, 2, 4, 0, 4],
                // 15. Turn figure (nota cambiata)
                &[0, 1, -1, 0, 1, 2],
                // 16. Sequence pattern (up 2, back 1)
                &[0, 1, 2, 1, 2, 3, 2],
                // 17. Wide arpeggio with return
                &[0, 2, 4, 2, 0, -2],
                // 18. Bach-style scalar sequence
                &[0, 1, 2, 3, 2, 1, 0, -1],
                // 19. Complex broken chord
                &[0, 2, 1, 3, 2, 4, 3, 5],
                // 20. Extended classical run with direction changes
                &[0, 1, 2, 3, 4, 3, 2, 1, 0, -1],
            ],

            // Blues: blue notes, bent notes, call-response, pentatonic licks
            // Sorted: simple bends → classic licks → complex turnarounds
            StylePattern::Blues => &[
                // 1. Repeat root (rhythmic emphasis)
                &[0, 0],
                // 2. Root to minor 3rd step
                &[0, 1],
                // 3. Simple up-down (root, next, root)
                &[0, 1, 0],
                // 4. Down step and back
                &[0, -1, 0],
                // 5. Three ascending
                &[0, 1, 2],
                // 6. Classic bend: root, up, back, down
                &[0, 1, 0, -1],
                // 7. Descending lick
                &[0, -1, -2, -1],
                // 8. Blues turnaround start
                &[0, -1, -2, 0],
                // 9. Ascending pentatonic fragment
                &[0, 1, 2, 3],
                // 10. Call-response (up, up, down, down)
                &[0, 1, 2, 0, -1],
                // 11. BB King lick (repeated top note with resolution)
                &[0, 2, 2, 1, 0],
                // 12. Descending blues run
                &[0, -1, -2, -3, -2],
                // 13. Classic blues lick with bend
                &[0, 1, 2, 1, 0, -1],
                // 14. Walking blues
                &[0, 1, 0, -1, 0, 1],
                // 15. Turnaround descending
                &[0, -1, -2, -3, -2, -1],
                // 16. Extended blues call
                &[0, 1, 2, 3, 2, 1, 0],
                // 17. Chicago blues lick
                &[0, 2, 1, 0, -1, 0, 1],
                // 18. Delta blues bend sequence
                &[0, 1, 0, 1, 2, 1, 0, -1],
                // 19. Blues scale run
                &[0, 1, 2, 3, 4, 3, 2, 1],
                // 20. Complex turnaround
                &[0, -1, -2, -3, -2, -1, 0, 1, 2],
            ],

            // Jazz: chord tones, extensions, bebop lines, chromatic approach
            // Sorted: simple intervals → chord arpeggios → bebop sequences
            StylePattern::Jazz => &[
                // 1. Root repetition
                &[0, 0],
                // 2. Up a third
                &[0, 1, 2],
                // 3. Down a third
                &[0, -1, -2],
                // 4. Enclosure (below, above, target)
                &[-1, 1, 0],
                // 5. Ascending 7th arpeggio (root, 3rd, 5th, 7th)
                &[0, 2, 4, 6],
                // 6. Descending 7th arpeggio
                &[0, -2, -4, -6],
                // 7. Up and back (3rds)
                &[0, 2, 0, -2],
                // 8. Enclosure with resolution
                &[-1, 1, 0, 1, 2],
                // 9. Ascending with skip
                &[0, 2, 1, 3, 2],
                // 10. Scale fragment with neighbor
                &[0, 1, 0, 2, 1],
                // 11. Bebop ascending
                &[0, 1, 2, 3, 4, 3],
                // 12. 1-2-3-5 jazz cliche
                &[0, 1, 2, 4, 2, 0],
                // 13. Honeysuckle rose figure
                &[0, 1, 0, -1, 0, 1, 2],
                // 14. Descending bebop line
                &[0, -1, -2, -1, -3, -2],
                // 15. Digital pattern (1-2-3-5)
                &[0, 1, 2, 4, 3, 2, 1],
                // 16. Coltrane-style wide intervals
                &[0, 3, 1, 4, 2, 5],
                // 17. Bebop scale run
                &[0, 1, 2, 3, 4, 5, 4, 3],
                // 18. Enclosure chain
                &[-1, 1, 0, 0, 2, 1, 3, 2],
                // 19. Parker-style bebop line
                &[0, 1, 2, 3, 2, 0, -1, 1, 0],
                // 20. Complex bebop sequence
                &[0, 2, 1, 3, 4, 3, 1, 0, -1, 0],
            ],

            // Rock: power chord shapes, pentatonic riffs, driving patterns
            // Sorted: simple power notes → riffs → complex sequences
            StylePattern::Rock => &[
                // 1. Root repetition (driving)
                &[0, 0],
                // 2. Root-fifth power
                &[0, 2],
                // 3. Root-octave
                &[0, 3],
                // 4. Down-up (palm mute style)
                &[0, 0, 1],
                // 5. Power chord arpeggio
                &[0, 2, 3],
                // 6. Ascending three
                &[0, 1, 2],
                // 7. Root pedal with neighbor
                &[0, 1, 0, -1],
                // 8. Descending riff
                &[0, -1, -2, 0],
                // 9. Classic rock ascend
                &[0, 1, 2, 3],
                // 10. Hammer-on pull-off figure
                &[0, 1, 0, 1, 0],
                // 11. Power chord sequence
                &[0, 2, 0, -2, 0],
                // 12. Pentatonic riff ascending
                &[0, 1, 2, 3, 2, 1],
                // 13. Gallop rhythm feel
                &[0, 0, 1, 0, 0, 2],
                // 14. Classic rock lick
                &[0, 2, 1, 0, -1, 0],
                // 15. Ascending riff with octave
                &[0, 1, 2, 3, 4, 3, 2],
                // 16. Descending power sequence
                &[0, -1, 0, -2, 0, -1, 0],
                // 17. Extended pentatonic
                &[0, 1, 2, 3, 4, 3, 2, 1],
                // 18. Rock sequence with pedal tone
                &[0, 2, 0, 3, 0, 2, 0, 1],
                // 19. Shred-style ascending
                &[0, 1, 2, 0, 1, 2, 3, 4],
                // 20. Complex rock run
                &[0, 1, 2, 3, 4, 5, 4, 3, 2, 1],
            ],

            // Latin: rhythmic arpeggios, montuno, syncopated patterns, tumbao
            // Sorted: simple → complex rhythmic/melodic figures
            StylePattern::Latin => &[
                // 1. Root repetition
                &[0, 0],
                // 2. Simple third
                &[0, 1],
                // 3. Root-fifth-root
                &[0, 2, 0],
                // 4. Ascending triad
                &[0, 1, 2],
                // 5. Tumbao bass (root, fifth, root)
                &[0, 4, 0],
                // 6. Simple montuno
                &[0, 1, 2, 1],
                // 7. Descending third
                &[0, -1, 0, 1],
                // 8. Ascending four
                &[0, 1, 2, 3],
                // 9. Bossa nova root movement
                &[0, 2, 1, 0, 2],
                // 10. Montuno figure
                &[0, 1, 2, 3, 2, 1],
                // 11. Tumbao extended
                &[0, -1, 0, 2, 0, -1],
                // 12. Son clave melodic
                &[0, 2, 1, 3, 0, 2],
                // 13. Salsa piano montuno
                &[0, 1, 2, 1, 3, 2, 1],
                // 14. Guajeo pattern
                &[0, 2, 1, 2, 0, 1, 0],
                // 15. Descending cha-cha
                &[0, -1, -2, -1, 0, 1, 2, 1],
                // 16. Extended montuno
                &[0, 1, 2, 3, 2, 1, 0, -1],
                // 17. Complex tumbao
                &[0, -1, 0, 2, 3, 2, 0, -1],
                // 18. Afro-Cuban sequence
                &[0, 1, 3, 2, 0, 1, 2, 3, 2],
                // 19. Salsa cascara melodic
                &[0, 2, 1, 0, 2, 3, 2, 1, 0],
                // 20. Complex Latin run
                &[0, 1, 2, 3, 4, 3, 2, 1, 0, -1],
            ],

            // Techno: repetitive, hypnotic, octave jumps, minimal variation
            // Sorted: static → rhythmic variation → complex sequences
            StylePattern::Techno => &[
                // 1. Root repeat
                &[0, 0],
                // 2. Octave up-down
                &[0, 3],
                // 3. Double root
                &[0, 0, 0],
                // 4. Root with octave hit
                &[0, 0, 3],
                // 5. Fifth accent
                &[0, 0, 2, 0],
                // 6. Minimal step
                &[0, 1, 0, 0],
                // 7. Acid pattern (up and back)
                &[0, 1, 0, -1, 0],
                // 8. Octave bounce
                &[0, 3, 0, 3, 0],
                // 9. Minimal arpeggio
                &[0, 1, 2, 0, 0],
                // 10. Root pedal with variation
                &[0, 0, 1, 0, 0, 2],
                // 11. 303-style acid
                &[0, 1, 2, 1, 0, -1],
                // 12. Ascending with return
                &[0, 0, 1, 1, 2, 0],
                // 13. Hypnotic sequence
                &[0, 1, 0, 2, 0, 1, 0],
                // 14. Techno stab sequence
                &[0, 2, 0, 0, 2, 0, -2],
                // 15. Acid ascending
                &[0, 1, 2, 3, 0, 1, 2, 0],
                // 16. Complex pedal pattern
                &[0, 0, 1, 0, 2, 0, 3, 0],
                // 17. Evolving sequence
                &[0, 1, 0, 2, 1, 3, 0, 2],
                // 18. Full acid run
                &[0, 1, 2, 3, 2, 1, 0, -1],
                // 19. Complex techno arpeggio
                &[0, 1, 2, 0, 3, 2, 1, 0, -1],
                // 20. Extended acid sequence
                &[0, 1, 0, 2, 0, 3, 2, 1, 0, -1],
            ],

            // Ambient: wide intervals, slow unfolding, open voicings, sustained
            // Sorted: simple intervals → wider movement → complex spacious figures
            StylePattern::Ambient => &[
                // 1. Unison
                &[0, 0],
                // 2. Step up
                &[0, 1],
                // 3. Fifth
                &[0, 2],
                // 4. Octave
                &[0, 3],
                // 5. Fifth and return
                &[0, 2, 0],
                // 6. Rising open
                &[0, 2, 3],
                // 7. Gentle descent
                &[0, -1, -2],
                // 8. Wide ascend
                &[0, 2, 4],
                // 9. Open triad
                &[0, 1, 3],
                // 10. Spacious return
                &[0, 2, 0, -2],
                // 11. Ascending open intervals
                &[0, 2, 3, 4],
                // 12. Drift up and settle
                &[0, 1, 2, 0],
                // 13. Wide pendulum
                &[0, 3, 0, -3, 0],
                // 14. Gentle wave
                &[0, 1, 2, 1, 0, -1],
                // 15. Spacious arpeggio
                &[0, 2, 4, 2, 0],
                // 16. Slow unfolding
                &[0, 1, 0, 2, 0, 3],
                // 17. Ethereal ascent
                &[0, 2, 1, 3, 2, 4],
                // 18. Wide arc
                &[0, 1, 2, 3, 4, 3, 2],
                // 19. Complex ambient drift
                &[0, 2, 0, 3, 1, 4, 2],
                // 20. Extended spacious journey
                &[0, 1, 3, 0, 2, 4, 1, 3],
            ],

            // Reggae: offbeat emphasis, root-fifth, skank patterns, dub spacing
            // Sorted: simple skank → rootsy patterns → complex dub sequences
            StylePattern::Reggae => &[
                // 1. Root repeat
                &[0, 0],
                // 2. Root-fifth
                &[0, 2],
                // 3. Root-fifth-root
                &[0, 2, 0],
                // 4. Simple skank up
                &[0, 1, 0],
                // 5. Root emphasis
                &[0, 0, 2, 0],
                // 6. Down and up
                &[0, -1, 0, 1],
                // 7. Reggae bass walk
                &[0, 1, 2, 0],
                // 8. Skank return
                &[0, 2, 1, 0],
                // 9. Dub step
                &[0, 0, 0, 2, 0],
                // 10. Roots ascending
                &[0, 1, 2, 1, 0],
                // 11. Offbeat emphasis
                &[0, -1, 0, 2, 0, -1],
                // 12. Bass-heavy walk
                &[0, -1, -2, -1, 0, 1],
                // 13. Skank sequence
                &[0, 1, 0, 2, 0, 1, 0],
                // 14. Dub spacing
                &[0, 0, 2, 0, 0, -2, 0],
                // 15. Roots melody
                &[0, 1, 2, 3, 2, 1, 0],
                // 16. Extended reggae walk
                &[0, -1, 0, 1, 2, 1, 0, -1],
                // 17. Dub echo pattern
                &[0, 2, 0, 2, 1, 0, -1, 0],
                // 18. Complex roots sequence
                &[0, 1, 0, -1, 0, 2, 1, 0],
                // 19. Steppers pattern
                &[0, 0, 1, 0, 2, 0, 1, 0, 0],
                // 20. Extended dub sequence
                &[0, 1, 2, 0, -1, 0, 2, 1, 0, -1],
            ],

            // Dubstep: sub-bass drops, octave dives, wobble patterns, glitch
            // Sorted: simple drops → wobble → complex glitch patterns
            StylePattern::Dubstep => &[
                // 1. Root repeat
                &[0, 0],
                // 2. Octave drop
                &[0, -3],
                // 3. Sub dive
                &[0, 0, -3],
                // 4. Octave bounce
                &[0, -3, 0],
                // 5. Double drop
                &[-3, 0, -3],
                // 6. Wobble root
                &[0, 0, 0, -1],
                // 7. Sub-bass climb
                &[-3, -2, -1, 0],
                // 8. Drop and climb
                &[0, -3, -2, -1],
                // 9. Stutter drop
                &[0, 0, -3, 0, 0],
                // 10. Wobble ascend
                &[0, -1, 0, 1, 0],
                // 11. Bass growl
                &[0, -1, 0, -2, 0, -1],
                // 12. Reese pattern
                &[0, -3, 0, -3, -2, 0],
                // 13. Glitch stutter
                &[0, 0, -1, 0, 0, -3, 0],
                // 14. Sub wobble
                &[0, -1, -2, -1, 0, 1, 0],
                // 15. Complex drop
                &[0, -2, -3, 0, -1, -3, 0],
                // 16. Glitch sequence
                &[0, 1, 0, -3, 0, 0, -2, 0],
                // 17. Riddim pattern
                &[0, 0, -1, 0, -3, 0, -1, 0],
                // 18. Bass design sequence
                &[0, -1, -2, -3, -2, -1, 0, 1],
                // 19. Complex wobble
                &[0, 1, 0, -1, -3, -1, 0, 1, 0],
                // 20. Extended glitch pattern
                &[0, 0, -3, 0, 1, 0, -3, -2, 0, 1],
            ],

            // Funk: syncopated, repetitive grooves, chromatic approach, rhythmic
            // Sorted: simple groove → syncopated → complex funk lines
            StylePattern::Funk => &[
                // 1. Root repeat
                &[0, 0],
                // 2. Octave slap
                &[0, 3],
                // 3. Root-octave-root
                &[0, 3, 0],
                // 4. Thumb-pop
                &[0, 0, 3, 0],
                // 5. Simple groove
                &[0, 1, 0, -1],
                // 6. Ascending funk
                &[0, 1, 2, 0],
                // 7. Slap pattern
                &[0, 3, 0, 3, 0],
                // 8. Chromatic approach
                &[-1, 0, 1, 0],
                // 9. Funk walk
                &[0, 1, 2, 1, 0],
                // 10. Syncopated groove
                &[0, 0, 2, 0, 1, 0],
                // 11. Bootsy pattern
                &[0, 3, 0, 1, 0, 3],
                // 12. Funk descend
                &[0, -1, 0, -2, 0, -1],
                // 13. Chromatic funk line
                &[0, -1, 0, 1, 2, 1, 0],
                // 14. Slap and pop extended
                &[0, 3, 0, 0, 2, 0, 3],
                // 15. Syncopated ascend
                &[0, 0, 1, 0, 2, 0, 1, 0],
                // 16. Parliament groove
                &[0, 2, 0, 3, 0, 2, 0, -1],
                // 17. Extended funk walk
                &[0, 1, 0, -1, 0, 2, 0, 1],
                // 18. Complex slap pattern
                &[0, 3, 2, 0, 3, 0, -1, 0],
                // 19. Funk sequence
                &[0, 1, 2, 0, -1, 0, 3, 0, 1],
                // 20. Extended funk groove
                &[0, 0, 2, 0, 3, 0, 1, 0, -1, 0],
            ],

            // Middle Eastern: augmented seconds, ornamental, maqam-based
            // Sorted: simple intervals → ornamental → complex maqam sequences
            StylePattern::MiddleEastern => &[
                // 1. Root repeat
                &[0, 0],
                // 2. Step up
                &[0, 1],
                // 3. Step up-down
                &[0, 1, 0],
                // 4. Ascending third
                &[0, 1, 2],
                // 5. Ornamental turn
                &[0, 1, 0, -1],
                // 6. Descending three
                &[0, -1, -2],
                // 7. Ascending four
                &[0, 1, 2, 3],
                // 8. Trill ornament
                &[0, 1, 0, 1, 0],
                // 9. Maqam ascend
                &[0, 1, 2, 1, 0],
                // 10. Descending four
                &[0, -1, -2, -3, -2],
                // 11. Extended trill
                &[0, 1, 0, 1, 0, 1],
                // 12. Maqam descent
                &[0, -1, -2, -1, 0, 1],
                // 13. Ascending with ornament
                &[0, 1, 0, 1, 2, 1, 2],
                // 14. Samai-style
                &[0, 1, 2, 3, 2, 1, 0],
                // 15. Wide maqam phrase
                &[0, 1, 2, 3, 4, 3, 2],
                // 16. Ornamental sequence
                &[0, 1, 0, 2, 1, 3, 2, 1],
                // 17. Maqam exploration
                &[0, -1, 0, 1, 2, 3, 2, 1],
                // 18. Taqsim-style improvisation
                &[0, 1, 2, 1, 0, -1, 0, 1, 2],
                // 19. Complex ornamental run
                &[0, 1, 0, 1, 2, 3, 2, 1, 0],
                // 20. Extended maqam journey
                &[0, 1, 2, 3, 4, 3, 2, 1, 0, -1],
            ],

            // Celtic: pentatonic, stepwise, jig/reel figures, grace notes
            // Sorted: simple steps → dance figures → complex reels
            StylePattern::Celtic => &[
                // 1. Root repeat
                &[0, 0],
                // 2. Step up
                &[0, 1],
                // 3. Step up-down
                &[0, 1, 0],
                // 4. Ascending three
                &[0, 1, 2],
                // 5. Grace note figure
                &[1, 0, 1],
                // 6. Descending three
                &[0, -1, -2],
                // 7. Jig figure (up-down-up)
                &[0, 1, 0, 1],
                // 8. Ascending four
                &[0, 1, 2, 3],
                // 9. Roll ornament
                &[0, 1, 0, -1, 0],
                // 10. Reel ascending
                &[0, 1, 2, 1, 0],
                // 11. Jig pattern
                &[0, 1, 2, 1, 0, -1],
                // 12. Hornpipe figure
                &[0, 1, 0, 2, 1, 0],
                // 13. Extended jig
                &[0, 1, 2, 3, 2, 1, 0],
                // 14. Slip jig (asymmetric)
                &[0, 1, 2, 0, 1, 2, 3],
                // 15. Reel sequence
                &[0, 1, 2, 3, 4, 3, 2, 1],
                // 16. Double grace note run
                &[0, 1, 0, 1, 2, 1, 2, 3],
                // 17. Descending reel
                &[0, -1, -2, -3, -2, -1, 0, 1],
                // 18. Complex jig pattern
                &[0, 1, 2, 1, 0, -1, 0, 1, 2],
                // 19. Extended reel
                &[0, 1, 2, 3, 2, 1, 0, -1, 0],
                // 20. Full Celtic run
                &[0, 1, 2, 3, 4, 3, 2, 1, 0, -1],
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum StyleMode {
    #[default]
    Replace,
    Finish,
}

impl StyleMode {
    pub fn all() -> &'static [StyleMode] {
        &[StyleMode::Replace, StyleMode::Finish]
    }

    pub fn name(&self) -> &'static str {
        match self {
            StyleMode::Replace => "Replace",
            StyleMode::Finish => "Finish",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleConfig {
    pub style: StylePattern,
    pub chance: u8,
    pub complexity: u8,
    pub max_notes: u8,
    #[serde(default)]
    pub mode: StyleMode,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            style: StylePattern::None,
            chance: 0,
            complexity: 10,
            max_notes: 4,
            mode: StyleMode::Replace,
        }
    }
}

impl StyleConfig {
    pub fn select_pattern<R: rand::Rng>(&self, rng: &mut R) -> Option<&'static [i8]> {
        if self.style == StylePattern::None || self.chance == 0 {
            return None;
        }

        let patterns = self.style.patterns();
        if patterns.is_empty() {
            return None;
        }

        let max_index = (self.complexity as usize).min(patterns.len());
        if max_index == 0 {
            return None;
        }

        let chance_roll: f32 = rng.gen();
        if chance_roll >= self.chance as f32 / 127.0 {
            return None;
        }

        let index = rng.gen_range(0..max_index);
        Some(patterns[index])
    }
}

/// Build a pitch sequence from a pattern and the available enabled notes.
/// `enabled_notes` must be sorted ascending by MIDI note.
/// `start_note` is the MIDI note of the beat that triggered the pattern.
/// `max_notes` limits how many notes to produce (loops pattern if exceeded).
/// Returns a Vec of MIDI notes to assign to consecutive beats.
pub fn build_pitch_sequence(
    pattern: &[i8],
    enabled_notes: &[u8],
    start_note: u8,
    max_notes: u8,
) -> Vec<u8> {
    if enabled_notes.is_empty() || pattern.is_empty() || max_notes == 0 {
        return vec![];
    }

    let start_index = find_nearest_index(enabled_notes, start_note);
    let count = max_notes as usize;

    (0..count).map(|i| {
        let step = pattern[i % pattern.len()];
        resolve_note(enabled_notes, start_index, step as i32)
    }).collect()
}

fn find_nearest_index(sorted_notes: &[u8], target: u8) -> usize {
    if sorted_notes.is_empty() {
        return 0;
    }

    let mut best_idx = 0;
    let mut best_dist = u8::MAX;

    for (i, &note) in sorted_notes.iter().enumerate() {
        let dist = note.abs_diff(target);
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }

    best_idx
}

fn resolve_note(sorted_notes: &[u8], start_index: usize, step: i32) -> u8 {
    let len = sorted_notes.len() as i32;
    if len == 0 {
        return 60;
    }

    let target_index = start_index as i32 + step;

    let octave_shift = if target_index < 0 {
        -1 + (target_index + 1) / len - if (target_index + 1) % len != 0 { 1 } else { 0 }
    } else if target_index >= len {
        target_index / len
    } else {
        0
    };

    let wrapped_index = ((target_index % len) + len) % len;
    let base_note = sorted_notes[wrapped_index as usize];
    let final_note = base_note as i32 + octave_shift * 12;

    final_note.clamp(0, 127) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_note_basic() {
        let notes = vec![48, 52, 55, 60]; // C3, E3, G3, C4
        assert_eq!(resolve_note(&notes, 0, 0), 48); // root
        assert_eq!(resolve_note(&notes, 0, 1), 52); // up 1 step
        assert_eq!(resolve_note(&notes, 0, 2), 55); // up 2 steps
        assert_eq!(resolve_note(&notes, 0, 3), 60); // up 3 steps
    }

    #[test]
    fn test_resolve_note_wraps_up() {
        let notes = vec![48, 52, 55, 60];
        // step 4 from index 0 = wrap to next octave of first note
        assert_eq!(resolve_note(&notes, 0, 4), 60); // 48 + 12
    }

    #[test]
    fn test_resolve_note_wraps_down() {
        let notes = vec![48, 52, 55, 60];
        // step -1 from index 0 = wrap down
        assert_eq!(resolve_note(&notes, 0, -1), 48); // 60 - 12
    }

    #[test]
    fn test_build_pitch_sequence() {
        let notes = vec![48, 52, 55, 60];
        let pattern: &[i8] = &[0, 1, 2];
        let result = build_pitch_sequence(pattern, &notes, 48, 3);
        assert_eq!(result, vec![48, 52, 55]);
    }

    #[test]
    fn test_build_pitch_sequence_loops() {
        let notes = vec![48, 52, 55, 60];
        let pattern: &[i8] = &[0, 1, 2];
        let result = build_pitch_sequence(pattern, &notes, 48, 6);
        assert_eq!(result, vec![48, 52, 55, 48, 52, 55]);
    }

    #[test]
    fn test_find_nearest_index() {
        let notes = vec![48, 52, 55, 60];
        assert_eq!(find_nearest_index(&notes, 48), 0);
        assert_eq!(find_nearest_index(&notes, 53), 1); // closer to 52
        assert_eq!(find_nearest_index(&notes, 60), 3);
    }

    #[test]
    fn test_all_styles_have_20_patterns() {
        for style in StylePattern::all() {
            if *style == StylePattern::None {
                continue;
            }
            assert_eq!(
                style.patterns().len(),
                20,
                "Style {:?} should have 20 patterns but has {}",
                style,
                style.patterns().len()
            );
        }
    }
}
