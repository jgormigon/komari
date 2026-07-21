use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use super::Bound;

/// A persistent model representing per-character configuration for a [`DailyQuestId`].
///
/// The navigation to reach the hunting ground and its hunting bound are fixed game content, not
/// user data - see [`DailyQuestId::navigation`] and [`DailyQuestId::bound`]. Only these fields
/// are user-editable; the mobbing key used for all daily quests is shared and lives on
/// [`super::Character::daily_quest_mobbing_key`] instead of per-entry.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DailyQuestEntry {
    pub id: DailyQuestId,
    pub kill_target: u32,
    pub enabled: bool,
    /// The UTC day index (see [`Self::today`]) this quest was last completed on, if any.
    ///
    /// Set by the tick loop through [`crate::ecs::CharacterUpdates`] when
    /// [`crate::rotator::DefaultRotator::rotate_daily_quest`] detects the kill quota reached, so
    /// an already-completed quest is skipped instead of re-run if the bot restarts or its
    /// actions get rebuilt again later the same day.
    #[serde(default)]
    pub last_completed_day: Option<u64>,
}

impl DailyQuestEntry {
    pub fn new(id: DailyQuestId) -> Self {
        Self {
            id,
            kill_target: 100,
            enabled: false,
            last_completed_day: None,
        }
    }

    /// The current UTC day index (days since the Unix epoch).
    ///
    /// This is the boundary daily quest completion is tracked against. Not adjusted for the game
    /// server's actual daily reset time, which may fall at a different point than UTC midnight -
    /// so a quest may appear completed for a few hours before/after the real in-game reset.
    pub fn today() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400
    }

    /// Whether this entry was already completed today (see [`Self::today`]).
    pub fn is_completed_today(&self) -> bool {
        self.last_completed_day == Some(Self::today())
    }
}

/// The world map's top-level region dropdown options.
#[derive(
    Clone, Copy, PartialEq, Default, Debug, Serialize, Deserialize, EnumIter, Display, EnumString,
)]
pub enum WorldMapRegion {
    #[default]
    #[strum(to_string = "Maple World")]
    MapleWorld,
    Grandis,
    #[strum(to_string = "Arcane River")]
    ArcaneRiver,
    Hielo,
}

/// A fixed catalog of known daily quest hunting grounds.
///
/// Unlike [`super::Map`], these aren't user-defined - the navigation to reach each one through
/// the in-game world map is fixed game content (see [`Self::navigation`]). Only the
/// per-character fields on [`DailyQuestEntry`] (enabled, kill target, hunting bound, mobbing key)
/// are user-editable.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumIter,
    Display,
    EnumString,
)]
/// Ordered as they should run - the daily quest solver runs entries in this declaration order
/// (via `Ord`), not the order they happen to be stored/added in.
pub enum DailyQuestId {
    #[strum(to_string = "Vanishing Journey")]
    VanishingJourney,
    #[strum(to_string = "Chu Chu Island")]
    ChuChuIsland,
    Lachelein,
    Arcana,
    Morass,
    Esfera,
    Moonbridge,
    #[strum(to_string = "Labyrinth of Suffering")]
    LabyrinthOfSuffering,
    Limina,
    Cernium,
    #[strum(to_string = "Hotel Arcus")]
    HotelArcus,
    Odium,
    #[strum(to_string = "Shangri-La")]
    ShangriLa,
    Arteria,
    Carcion,
    Tallahart,
}

/// Fixed navigation data for a [`DailyQuestId`] - see [`DailyQuestId::navigation`].
#[derive(Clone, Debug)]
pub struct DailyQuestNavigation {
    pub region: WorldMapRegion,
    pub dropdown_path: Vec<String>,
    pub location_label: String,
    pub location_point: (i32, i32),
    pub sub_location_label: Option<String>,
}

impl DailyQuestId {
    /// The fixed world map navigation for this quest, captured empirically from reference
    /// screenshots.
    ///
    /// - Pick `region` from the top dropdown, then each entry of `dropdown_path` in order from
    ///   the next dropdown(s) it reveals (e.g. `["Tenebris", "Moonbridge"]` to drill from Arcane
    ///   River down to Tenebris's Moonbridge sub-map). Every dropdown's option list is
    ///   always-visible text, found via OCR.
    /// - Once there, double-click the hunting ground's node. When `dropdown_path` is empty, the
    ///   target is directly visible as a labelled node/banner on the region's own top-level
    ///   overview, found via OCR against `location_label`. When `dropdown_path` is non-empty, the
    ///   deeper map it leads to only shows individual room names as a hover tooltip rather than
    ///   always-visible text - OCR can't find those, so `location_point` (a pixel offset from the
    ///   world map's anchor) is used to click directly instead, and `location_label` is then just
    ///   a human-readable name for display.
    /// - Some nodes (e.g. two hunting grounds sharing one icon) lead to an intermediate view
    ///   instead of an immediate teleport prompt - `sub_location_label`, if set, is
    ///   double-clicked there to reach the actual target.
    pub fn navigation(self) -> DailyQuestNavigation {
        fn nav(
            region: WorldMapRegion,
            dropdown_path: &[&str],
            location_label: &str,
            location_point: (i32, i32),
            sub_location_label: Option<&str>,
        ) -> DailyQuestNavigation {
            DailyQuestNavigation {
                region,
                dropdown_path: dropdown_path.iter().map(|path| path.to_string()).collect(),
                location_label: location_label.to_string(),
                location_point,
                sub_location_label: sub_location_label.map(str::to_string),
            }
        }

        use WorldMapRegion::{ArcaneRiver, Grandis};

        match self {
            DailyQuestId::VanishingJourney => nav(
                ArcaneRiver,
                &["Vanishing Journey"],
                "Extinction Zone : Spirit Zone",
                (389, 301),
                None,
            ),
            DailyQuestId::ChuChuIsland => nav(
                ArcaneRiver,
                &["Chu Chu Island"],
                "Slurpy Forest : Bitty-Bobble Forest 1",
                (434, 421),
                None,
            ),
            DailyQuestId::Lachelein => nav(
                ArcaneRiver,
                &["Lachelein, the Dreaming City"],
                "Lachelein Ballroom : Revelation Place 3",
                (544, 418),
                None,
            ),
            DailyQuestId::Arcana => nav(
                ArcaneRiver,
                &["Arcana, The Mysterious Forest"],
                "Arcana : Cavern Lower Path",
                (461, 412),
                None,
            ),
            DailyQuestId::Morass => nav(
                ArcaneRiver,
                &["Morass, Swamp of Memory"],
                "Morass : Shadowdance Hall 4",
                (516, 198),
                None,
            ),
            DailyQuestId::Esfera => nav(
                ArcaneRiver,
                &["Esfera, The Origin Sea"],
                "Esfera : Mirror-touched Sea 3",
                (559, 452),
                None,
            ),
            DailyQuestId::Moonbridge => nav(
                ArcaneRiver,
                &["Tenebris", "Moonbridge"],
                "Moonbridge : Void Current 3",
                (579, 458),
                None,
            ),
            DailyQuestId::LabyrinthOfSuffering => nav(
                ArcaneRiver,
                &["Tenebris", "Labyrinth of Suffering"],
                "Tenebris : Labyrinth of Suffering Deep Core 1",
                (263, 463),
                None,
            ),
            DailyQuestId::Limina => nav(
                ArcaneRiver,
                &["Tenebris", "Limina"],
                "Limina : End of the World 2-6",
                (584, 441),
                None,
            ),
            DailyQuestId::Cernium => nav(
                Grandis,
                &["Western Grandis", "Cernium"],
                "Cernium : Royal Library Section 1",
                (511, 148),
                None,
            ),
            DailyQuestId::HotelArcus => nav(
                Grandis,
                &["Western Grandis", "Hotel Arcus"],
                "Hotel Arcus : Nostalgic Drive-in Theater 4",
                (494, 315),
                None,
            ),
            DailyQuestId::Odium => nav(
                Grandis,
                &["Western Grandis", "Odium"],
                "Odium : Captured Alley 2",
                (426, 281),
                None,
            ),
            DailyQuestId::ShangriLa => nav(
                Grandis,
                &["Western Grandis", "Shangri-La"],
                "Shangri-La : Blooming Spring 2",
                (321, 473),
                None,
            ),
            DailyQuestId::Arteria => nav(
                Grandis,
                &["Western Grandis", "Arteria"],
                "Empress Road : Southern Outskirts",
                (213, 455),
                None,
            ),
            DailyQuestId::Carcion => nav(
                Grandis,
                &["Western Grandis", "Carcion"],
                "Carcion : Giant Coral Colony 3",
                (198, 171),
                None,
            ),
            DailyQuestId::Tallahart => nav(
                Grandis,
                &["Western Grandis", "Tallahart"],
                "Tallahart : Silent Ashlands 3",
                (184, 461),
                None,
            ),
        }
    }

    /// The fixed hunting bound for this quest's map, as provided by the user.
    pub fn bound(self) -> Bound {
        match self {
            DailyQuestId::VanishingJourney => Bound {
                x: 18,
                y: 11,
                width: 129,
                height: 41,
            },
            DailyQuestId::ChuChuIsland => Bound {
                x: 25,
                y: 7,
                width: 110,
                height: 34,
            },
            DailyQuestId::Lachelein => Bound {
                x: 25,
                y: 10,
                width: 120,
                height: 30,
            },
            DailyQuestId::Arcana => Bound {
                x: 26,
                y: 8,
                width: 115,
                height: 45,
            },
            DailyQuestId::Morass => Bound {
                x: 3,
                y: 15,
                width: 203,
                height: 50,
            },
            DailyQuestId::Esfera => Bound {
                x: 13,
                y: 20,
                width: 142,
                height: 50,
            },
            DailyQuestId::Moonbridge => Bound {
                x: 15,
                y: 18,
                width: 145,
                height: 48,
            },
            DailyQuestId::LabyrinthOfSuffering => Bound {
                x: 40,
                y: 10,
                width: 169,
                height: 61,
            },
            DailyQuestId::Limina => Bound {
                x: 34,
                y: 44,
                width: 100,
                height: 38,
            },
            DailyQuestId::Cernium => Bound {
                x: 12,
                y: 17,
                width: 141,
                height: 44,
            },
            DailyQuestId::HotelArcus => Bound {
                x: 26,
                y: 20,
                width: 145,
                height: 38,
            },
            DailyQuestId::Odium => Bound {
                x: 7,
                y: 18,
                width: 157,
                height: 40,
            },
            DailyQuestId::ShangriLa => Bound {
                x: 4,
                y: 11,
                width: 174,
                height: 49,
            },
            DailyQuestId::Arteria => Bound {
                x: 7,
                y: 31,
                width: 153,
                height: 44,
            },
            DailyQuestId::Carcion => Bound {
                x: 7,
                y: 15,
                width: 167,
                height: 45,
            },
            DailyQuestId::Tallahart => Bound {
                x: 10,
                y: 23,
                width: 166,
                height: 45,
            },
        }
    }
}
