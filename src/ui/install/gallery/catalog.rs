// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Born2BSalty

use crate::app::modlist_share::encode_share_payload_text;
use crate::registry::model::Game;

pub struct GalleryMod {
    pub mod_name: &'static str,
    pub tp_file: &'static str,
    pub component_id: &'static str,
    pub component_label: &'static str,
    pub target: Game,
    pub wlb_inputs: Option<&'static str>,
}

pub struct GalleryEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub author: &'static str,
    pub game: Game,
    pub tags: &'static [&'static str],
    pub starter: bool,
    pub sample: bool,
    pub description: &'static str,
    pub requirements: &'static str,
    pub version: &'static str,
    pub mods: &'static [GalleryMod],
    pub payload: &'static str,
}

impl GalleryEntry {
    #[must_use]
    pub fn mod_count(&self) -> usize {
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for gallery_mod in self.mods {
            let key = (gallery_mod.mod_name, gallery_mod.tp_file);
            if !seen.contains(&key) {
                seen.push(key);
            }
        }
        seen.len()
    }
}

const BIO_TEAM: &str = "BIO Team";
const COMMUNITY: &str = "Community Collection";

const REQUIREMENTS_EET: &str =
    "Baldur's Gate: Enhanced Edition and Baldur's Gate II: Enhanced Edition";
const REQUIREMENTS_BGEE: &str = "Baldur's Gate: Enhanced Edition";
const REQUIREMENTS_BGEE_SOD: &str =
    "Baldur's Gate: Enhanced Edition with the Siege of Dragonspear DLC";
const REQUIREMENTS_BG2EE: &str = "Baldur's Gate II: Enhanced Edition";
const REQUIREMENTS_IWDEE: &str = "Icewind Dale: Enhanced Edition";
const REQUIREMENTS_EET_WINDOWS: &str =
    "Baldur's Gate: Enhanced Edition and Baldur's Gate II: Enhanced Edition, Windows only (EEex)";

#[must_use]
pub(crate) const fn requirements_for(game: Game) -> &'static str {
    match game {
        Game::BGEE => REQUIREMENTS_BGEE,
        Game::BG2EE => REQUIREMENTS_BG2EE,
        Game::IWDEE => REQUIREMENTS_IWDEE,
        Game::EET => REQUIREMENTS_EET,
    }
}

const EET_BG1_FOLDER_PROMPT: &str = r"y,C:\BIO\Baldur's Gate Enhanced Edition";

const ENTRIES: &[GalleryEntry] = &[
    GalleryEntry {
        id: "eet-essentials",
        name: "EET Essentials",
        author: BIO_TEAM,
        game: Game::EET,
        tags: &["Starter", "Vanilla+", "Quality of life"],
        starter: true,
        sample: true,
        description: "The EET spine with the rough edges filed off: merge the campaigns, bridge Baldur's Gate into Shadows of Amn, then the community fixpack, EEex, a cleaner UI, the graphical overhaul, Icewind Dale's spells and a light pass of quality-of-life tweaks that never reshape the saga.",
        requirements: REQUIREMENTS_EET_WINDOWS,
        version: "1.0.0",
        mods: &[
            GalleryMod {
                mod_name: "DlcMerger",
                tp_file: "DLCMERGER.TP2",
                component_id: "1",
                component_label: "Merge DLC into game -> Siege of Dragonspear",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "0",
                component_label: "Core Fixes",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "2",
                component_label: "Game Text Update",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "0",
                component_label: "Core Fixes",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "2",
                component_label: "Game Text Update",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET",
                tp_file: "EET.TP2",
                component_id: "0",
                component_label: "EET core (resource importation)",
                target: Game::BG2EE,
                wlb_inputs: Some(EET_BG1_FOLDER_PROMPT),
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "0",
                component_label: "Quick Menu Core",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "1",
                component_label: "EEex",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "2",
                component_label: "Enable effect menu module: LShift-on-hover to view spells affecting creature",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "3",
                component_label: "Enable empty container module: Highlight empty containers in gray instead of cyan",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "4",
                component_label: "Enable hotkey module: Edit override/B3Hotkey.lua to create advanced spell hotkeys",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "5",
                component_label: "Enable scale module: Customizable UI scaling factor",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "6",
                component_label: "Enable time step module: Advance 1 game tick on keypress",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEex",
                tp_file: "EEEX.TP2",
                component_id: "7",
                component_label: "Enable timer module: Visual indicators for modal actions, contingencies, and spell/item cooldowns",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "LeUI",
                tp_file: "LEUI.TP2",
                component_id: "0",
                component_label: "lefreut's Enhanced UI - Core component",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEUITweaks",
                tp_file: "EEUITWEAKS.TP2",
                component_id: "1070",
                component_label: "Faydark's Abilities Auto-Roller/GrimLefourbe's BG2 UI",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEUITweaks",
                tp_file: "EEUITWEAKS.TP2",
                component_id: "1100",
                component_label: "Display max proficiency limits",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "bubb_spell_menu_extended",
                tp_file: "BUBB_SPELL_MENU_EXTENDED.TP2",
                component_id: "0",
                component_label: "Bubb's Spell Menu Extended",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "BGGO",
                tp_file: "BGGO.TP2",
                component_id: "0",
                component_label: "Baldurs Gate Graphical Overhaul Core",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HQ_SoundClips_BG2EE",
                tp_file: "HQ_SOUNDCLIPS_BG2EE.TP2",
                component_id: "0",
                component_label: "Install high quality soundclips for new BG2EE content",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "IWDification",
                tp_file: "SETUP-IWDIFICATION.TP2",
                component_id: "10",
                component_label: "Icewind Dale Casting Graphics (Andyr)",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "IWDification",
                tp_file: "SETUP-IWDIFICATION.TP2",
                component_id: "30",
                component_label: "IWD Arcane Spell Pack",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "IWDification",
                tp_file: "SETUP-IWDIFICATION.TP2",
                component_id: "40",
                component_label: "IWD Divine Spell Pack",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "130",
                component_label: "Force All Dialogue to Pause Game",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "140",
                component_label: "Fix Boo's Squeak",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "170",
                component_label: "Unique Icons [Lava] -> Only replace icons that aren't already unique",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "182",
                component_label: "Unique Containers [Miloch] -> Unique icons and names",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "191",
                component_label: "Use Character Colors Instead of Item Colors -> For non-magical shields and helmets",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "250",
                component_label: "Colorize NPC Names and Tooltips -> Normal brightness",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "1075",
                component_label: "Send BioWare NPCs to an Inn [DavidW/Zed Nocear]",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "1080",
                component_label: "Add Bags of Holding",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "1120",
                component_label: "Stores Sell Higher Stacks of Items",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "2780",
                component_label: "P&P Free Action",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "2999",
                component_label: "Max HP at Level One",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3000",
                component_label: "Higher HP on Level Up -> Maximum",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3010",
                component_label: "Maximum HP Creatures [the bigg] -> For all creatures in game",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3030",
                component_label: "Easy Spell Learning -> 100% learn spells",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3040",
                component_label: "Make Bags of Holding Bottomless",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3080",
                component_label: "Increase Ammo Stack Size -> Unlimited ammo stacking",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3090",
                component_label: "Increase Jewelry, Gem, and Miscellaneous Item Stacks -> Unlimited jewelry, gem, and miscellaneous item stacking",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3100",
                component_label: "Increase Potion Stacking -> Unlimited potion stacking",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3110",
                component_label: "Increase Scroll Stacking -> Unlimited scroll stacking",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "3354",
                component_label: "Create Interval Saves [argent77] -> Every 15 minutes (cycle through four saves)",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4000",
                component_label: "Adjust Evil Joinable NPC Reaction Rolls",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4025",
                component_label: "Allow NPC Pairs to Separate",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4031",
                component_label: "Consistent Stats: Edwin -> Use BG2 values",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4041",
                component_label: "Consistent Stats: Jaheira -> Use BG2 values",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4050",
                component_label: "Change Jaheira to Neutral Good Alignment",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4061",
                component_label: "Consistent Stats: Minsc -> Use BG2 values",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4071",
                component_label: "Consistent Stats: Viconia -> Use BG2 values",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4150",
                component_label: "Move Boo Into Minsc's Pack",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "4170",
                component_label: "Ensure Shar-Teel Doesn't Die in the Original Challenge",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "10",
                component_label: "Add in-game option \"Enable Debug Mode\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "12",
                component_label: "Add in-game option \"Show Strrefs\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "13",
                component_label: "Add in-game option \"Hotkeys On Tooltips\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "33",
                component_label: "Add in-game option \"Enhanced Path Search\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "14",
                component_label: "Add in-game option \"Show trigger icons on tab\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "16",
                component_label: "Add in-game option \"Limit druidic spells for Cleric/Ranger\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "17",
                component_label: "Add in-game option \"3E Sneak Attack\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "18",
                component_label: "Add in-game option \"Critical Hit Screen Shake\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "19",
                component_label: "Add in-game option \"Show extra combat info\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "20",
                component_label: "Add in-game option \"Show Game Date and Time on Pause\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "23",
                component_label: "Add in-game option \"Pause Game on Map Screen\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "25",
                component_label: "Add in-game option \"Disable Movies\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "27",
                component_label: "Add in-game option \"XP Bonus in Nightmare Mode\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "32",
                component_label: "Add in-game option \"Show Area of Effect Range\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "35",
                component_label: "Add in-game option \"Show Learnable Spells\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "36",
                component_label: "Add in-game option \"Render Search Map\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "37",
                component_label: "Add in-game option \"Render Dynamic Search Map\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "38",
                component_label: "Add in-game options for Tweak Anthology's \"Create Interval Saves\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "39",
                component_label: "Add in-game option \"Force Dialog Pause\"",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "HiddenGameplayOptions",
                tp_file: "HIDDENGAMEPLAYOPTIONS.TP2",
                component_id: "200",
                component_label: "Improved Cheat Menu",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "remastered_spell_icons",
                tp_file: "REMASTERED_SPELL_ICONS.TP2",
                component_id: "0",
                component_label: "Install Remastered Spell Icons Core Component",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "remastered_spell_icons",
                tp_file: "REMASTERED_SPELL_ICONS.TP2",
                component_id: "1",
                component_label: "Use IWD:EE Colors (Green icons for summoning spells)",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "2042",
                component_label: "XP for Traps, Spells and Lockpicking -> Vanilla friendly progressive",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "3000",
                component_label: "Disable hostile reaction after charm",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "3022",
                component_label: "Familiar death consequences -> Disabled",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "4020",
                component_label: "Higher framerates support",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "4040",
                component_label: "Import party items to SoA",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "4050",
                component_label: "Books/Scrolls categorization",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "4060",
                component_label: "Wand Case",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_Tweaks",
                tp_file: "EET_TWEAKS.TP2",
                component_id: "4070",
                component_label: "Key Ring",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET_end",
                tp_file: "EET_END.TP2",
                component_id: "0",
                component_label: "EET end (last mod in install order)",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
        ],
        payload: include_str!("catalog/eet-essentials.json"),
    },
    GalleryEntry {
        id: "eet-plus-fixes",
        name: "EET + Fixes",
        author: BIO_TEAM,
        game: Game::EET,
        tags: &["Starter", "Fixes"],
        starter: true,
        sample: true,
        description: "EET Essentials with the community fixpack layered on top, so the merged saga starts from a patched engine instead of a vanilla one.",
        requirements: REQUIREMENTS_EET,
        version: "1.0.0",
        mods: &[
            GalleryMod {
                mod_name: "DlcMerger",
                tp_file: "DLCMERGER.TP2",
                component_id: "1",
                component_label: "Merge DLC into game -> Siege of Dragonspear",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "0",
                component_label: "Core Fixes",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "2",
                component_label: "Game Text Update",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "0",
                component_label: "Core Fixes",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "2",
                component_label: "Game Text Update",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EET",
                tp_file: "EET.TP2",
                component_id: "0",
                component_label: "EET core (resource importation)",
                target: Game::BG2EE,
                wlb_inputs: Some(EET_BG1_FOLDER_PROMPT),
            },
            GalleryMod {
                mod_name: "EET_end",
                tp_file: "EET_END.TP2",
                component_id: "0",
                component_label: "EET end (last mod in install order)",
                target: Game::BG2EE,
                wlb_inputs: None,
            },
        ],
        payload: include_str!("catalog/eet-plus-fixes.json"),
    },
    GalleryEntry {
        id: "bgee-vanilla-plus",
        name: "BGEE Vanilla+ (with DLC)",
        author: COMMUNITY,
        game: Game::BGEE,
        tags: &["Vanilla+", "Quality of life", "Siege of Dragonspear"],
        starter: false,
        sample: true,
        description: "Baldur's Gate as it shipped, minus the rough edges: Siege of Dragonspear merged in first, then the community fixpack and a light pass of quality-of-life tweaks.",
        requirements: REQUIREMENTS_BGEE_SOD,
        version: "1.0.0",
        mods: &[
            GalleryMod {
                mod_name: "DlcMerger",
                tp_file: "DLCMERGER.TP2",
                component_id: "1",
                component_label: "Merge DLC into game -> Siege of Dragonspear",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "0",
                component_label: "Core Fixes",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "2",
                component_label: "Game Text Update",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "CDTweaks",
                tp_file: "SETUP-CDTWEAKS.TP2",
                component_id: "2010",
                component_label: "Increase Ammo Stacking",
                target: Game::BGEE,
                wlb_inputs: None,
            },
        ],
        payload: include_str!("catalog/bgee-vanilla-plus.json"),
    },
    GalleryEntry {
        id: "bgee-vanilla-plus-no-dlc",
        name: "BGEE Vanilla+ (no DLC)",
        author: COMMUNITY,
        game: Game::BGEE,
        tags: &["Vanilla+", "Fixes only"],
        starter: false,
        sample: true,
        description: "Baldur's Gate as it shipped, minus the rough edges, for a game without the Siege of Dragonspear archive: the community fixpack only.",
        requirements: REQUIREMENTS_BGEE,
        version: "1.0.0",
        mods: &[
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "0",
                component_label: "Core Fixes",
                target: Game::BGEE,
                wlb_inputs: None,
            },
            GalleryMod {
                mod_name: "EEFixPack",
                tp_file: "SETUP-EEFIXPACK.TP2",
                component_id: "2",
                component_label: "Game Text Update",
                target: Game::BGEE,
                wlb_inputs: None,
            },
        ],
        payload: include_str!("catalog/bgee-vanilla-plus-no-dlc.json"),
    },
    GalleryEntry {
        id: "iwdee-essentials",
        name: "Icewind Dale Essentials",
        author: COMMUNITY,
        game: Game::IWDEE,
        tags: &["Starter", "Quality of life"],
        starter: true,
        sample: true,
        description: "A short, safe starting point for Icewind Dale: the tweaks most players turn on first, and nothing that reshapes the campaign.",
        requirements: REQUIREMENTS_IWDEE,
        version: "1.0.0",
        mods: &[GalleryMod {
            mod_name: "CDTweaks",
            tp_file: "SETUP-CDTWEAKS.TP2",
            component_id: "2010",
            component_label: "Increase Ammo Stacking",
            target: Game::IWDEE,
            wlb_inputs: None,
        }],
        payload: include_str!("catalog/iwdee-essentials.json"),
    },
];

#[must_use]
pub const fn entries() -> &'static [GalleryEntry] {
    ENTRIES
}

pub fn share_code(entry: &GalleryEntry) -> Result<String, String> {
    encode_share_payload_text(entry.payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::modlist_share::preview_modlist_share_code;

    #[test]
    fn mod_count_counts_distinct_mods_not_components() {
        let by_id = |id: &str| {
            entries()
                .iter()
                .find(|e| e.id == id)
                .expect("entry is in the catalog")
        };
        assert_eq!(by_id("eet-plus-fixes").mod_count(), 4);
        assert_eq!(by_id("bgee-vanilla-plus").mod_count(), 3);
        assert_eq!(by_id("bgee-vanilla-plus-no-dlc").mod_count(), 1);
        assert_eq!(by_id("eet-essentials").mod_count(), 15);
        assert_eq!(by_id("iwdee-essentials").mod_count(), 1);
    }

    #[test]
    fn catalog_holds_the_five_stub_entries() {
        let names: Vec<&str> = entries().iter().map(|e| e.name).collect();
        assert_eq!(
            names,
            vec![
                "EET Essentials",
                "EET + Fixes",
                "BGEE Vanilla+ (with DLC)",
                "BGEE Vanilla+ (no DLC)",
                "Icewind Dale Essentials",
            ]
        );
    }

    #[test]
    fn entry_ids_are_unique() {
        let mut ids: Vec<&str> = entries().iter().map(|e| e.id).collect();
        ids.sort_unstable();
        let total = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), total, "gallery ids must be unique");
    }

    #[test]
    fn every_entry_is_flagged_sample_with_a_version() {
        for entry in entries() {
            assert!(entry.sample, "{} must carry the sample flag", entry.name);
            assert_eq!(entry.version, "1.0.0");
            assert!(!entry.mods.is_empty());
            assert!(!entry.description.trim().is_empty());
            assert!(!entry.requirements.trim().is_empty());
        }
    }

    #[test]
    fn three_entries_are_starter_lists() {
        assert_eq!(entries().iter().filter(|e| e.starter).count(), 3);
    }

    #[test]
    fn every_entry_generates_a_code_that_parses_back_with_its_provenance() {
        for entry in entries() {
            let code = share_code(entry).expect("catalog entry must export a share code");
            let preview =
                preview_modlist_share_code(&code).expect("generated code must parse back");
            assert_eq!(preview.game_install, entry.game.to_legacy_string());
            assert_eq!(preview.name.as_deref(), Some(entry.name));
            assert_eq!(preview.author.as_deref(), Some(entry.author));
            assert!(
                preview.allow_auto_install,
                "{} must permit install-as-provided",
                entry.name
            );
        }
    }

    #[test]
    fn eet_essentials_splits_three_bgee_and_eighty_bg2ee_entries() {
        let entry = entries()
            .iter()
            .find(|e| e.id == "eet-essentials")
            .expect("EET Essentials is in the catalog");
        let code = share_code(entry).expect("export");
        let preview = preview_modlist_share_code(&code).expect("parse");
        assert_eq!(preview.bgee_entries, 3);
        assert_eq!(preview.bg2ee_entries, 80);
    }

    #[test]
    fn every_entry_carries_source_overrides_and_no_installed_refs_or_mod_configs() {
        for entry in entries() {
            let code = share_code(entry).expect("catalog entry must export a share code");
            let preview =
                preview_modlist_share_code(&code).expect("generated code must parse back");
            assert!(
                preview.has_source_overrides,
                "{} must carry source overrides",
                entry.name
            );
            assert!(
                !preview.has_installed_refs,
                "{} must carry no installed refs",
                entry.name
            );
            assert_eq!(
                preview.mod_config_count, 0,
                "{} must carry no mod configs",
                entry.name
            );
        }
    }

    const EET_ESSENTIALS_PINS: [(&str, &str, &str, &str, &str); 12] = [
        (
            "DlcMerger",
            "commit",
            "bfd167f7a52dfa6c9e694955a074a85991b0c358",
            "argent77",
            "Argent77",
        ),
        (
            "eefixpack",
            "commit",
            "9db254fe0c046d789381de0022ff37eb2c3a3979",
            "gibberlings3",
            "Gibberlings3",
        ),
        (
            "eet",
            "commit",
            "b164f5997de7a00ef69177d8221c19bd1e34cf10",
            "gibberlings3",
            "Gibberlings3",
        ),
        (
            "EET_END",
            "commit",
            "b164f5997de7a00ef69177d8221c19bd1e34cf10",
            "gibberlings3",
            "Gibberlings3",
        ),
        (
            "cdtweaks",
            "commit",
            "7649ced6cd25865874d787ec1a9abbc67b068729",
            "gibberlings3",
            "Gibberlings3",
        ),
        (
            "HiddenGameplayOptions",
            "commit",
            "ae8571b19d8a157b13367bfe7d379d2dfb456d75",
            "argent77",
            "Argent77",
        ),
        (
            "HQ_SoundClips_BG2EE",
            "commit",
            "17b4444d8fb8fd3324648027a8119045bd6c239a",
            "argent77",
            "Argent77",
        ),
        (
            "LeUI",
            "commit",
            "0e3c400ca4a85cd84933b77da0b22c6c5c322ef2",
            "r-e-d",
            "r-e-d",
        ),
        (
            "remastered_spell_icons",
            "commit",
            "14d4568b326ce04e767815a61489a89c51033b76",
            "renegade0",
            "Renegade0",
        ),
        ("eeex", "tag", "v1.2.0", "bubb13", "Bubb13"),
        (
            "bubb_spell_menu_extended",
            "tag",
            "v5.2",
            "bubb13",
            "Bubb13",
        ),
        ("EET_Tweaks", "tag", "v1.12", "k4thos", "K4thos"),
    ];

    fn overrides_text_for(id: &str) -> String {
        let entry = entries()
            .iter()
            .find(|e| e.id == id)
            .expect("entry is in the catalog");
        let code = share_code(entry).expect("export");
        preview_modlist_share_code(&code)
            .expect("parse")
            .source_overrides_text
    }

    fn overrides_mod_block_count(overrides: &str) -> usize {
        toml::from_str::<toml::Value>(overrides)
            .ok()
            .and_then(|value| {
                value
                    .get("mods")
                    .and_then(toml::Value::as_array)
                    .map(Vec::len)
            })
            .unwrap_or(0)
    }

    fn assert_pins(overrides: &str, expected: &[(&str, &str, &str, &str, &str)], label: &str) {
        use crate::app::mod_downloads::{SourceTier, source_tiers_from_texts};

        assert_eq!(
            overrides_mod_block_count(overrides),
            expected.len(),
            "{label}: unexpected number of [[mods]] blocks in overrides"
        );

        let tiers = source_tiers_from_texts(
            include_str!("../../../core/config/default_mod_downloads.toml"),
            "",
            overrides,
        );
        for (tp2, kind, value, source_id, source_label) in expected {
            let (source, tier) = tiers
                .resolve(tp2)
                .unwrap_or_else(|| panic!("{label}: {tp2} must resolve"));
            assert_eq!(
                tier,
                SourceTier::Modlist,
                "{label}: {tp2} must resolve from the modlist overlay, not the stock tier"
            );
            assert_eq!(
                &source.source_id, source_id,
                "{label}: {tp2} source id mismatch"
            );
            assert_eq!(
                &source.source_label, source_label,
                "{label}: {tp2} source label mismatch"
            );
            assert!(
                source.channel.is_none(),
                "{label}: {tp2} must carry no channel"
            );
            match *kind {
                "commit" => {
                    let commit = source.commit.as_deref().unwrap_or_default();
                    assert_eq!(
                        commit.len(),
                        40,
                        "{label}: {tp2} commit must be 40 hex characters"
                    );
                    assert!(
                        commit.chars().all(|c| c.is_ascii_hexdigit()),
                        "{label}: {tp2} commit must be hex"
                    );
                    assert_eq!(commit, *value, "{label}: {tp2} commit mismatch");
                    assert!(
                        source.tag.is_none(),
                        "{label}: {tp2} must carry only a commit"
                    );
                    assert!(
                        source.branch.is_none(),
                        "{label}: {tp2} must carry only a commit"
                    );
                }
                "tag" => {
                    assert_eq!(
                        source.tag.as_deref(),
                        Some(*value),
                        "{label}: {tp2} tag mismatch"
                    );
                    assert!(
                        source.commit.is_none(),
                        "{label}: {tp2} must carry only a tag"
                    );
                    assert!(
                        source.branch.is_none(),
                        "{label}: {tp2} must carry only a tag"
                    );
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn every_source_snapshot_pin_is_a_full_commit_sha() {
        assert_pins(
            &overrides_text_for("eet-essentials"),
            &EET_ESSENTIALS_PINS,
            "eet-essentials",
        );

        let subset = |tp2s: &[&str]| -> Vec<(&str, &str, &str, &str, &str)> {
            EET_ESSENTIALS_PINS
                .iter()
                .filter(|(tp2, ..)| tp2s.contains(tp2))
                .copied()
                .collect()
        };

        assert_pins(
            &overrides_text_for("eet-plus-fixes"),
            &subset(&["DlcMerger", "eefixpack", "eet", "EET_END"]),
            "eet-plus-fixes",
        );
        assert_pins(
            &overrides_text_for("bgee-vanilla-plus"),
            &subset(&["DlcMerger", "eefixpack", "cdtweaks"]),
            "bgee-vanilla-plus",
        );
        assert_pins(
            &overrides_text_for("bgee-vanilla-plus-no-dlc"),
            &subset(&["eefixpack"]),
            "bgee-vanilla-plus-no-dlc",
        );
        assert_pins(
            &overrides_text_for("iwdee-essentials"),
            &subset(&["cdtweaks"]),
            "iwdee-essentials",
        );
    }

    #[test]
    fn eet_essentials_order_holds_the_reference_spine() {
        let entry = entries()
            .iter()
            .find(|e| e.id == "eet-essentials")
            .expect("EET Essentials is in the catalog");
        let first_game_mods: Vec<&GalleryMod> = entry
            .mods
            .iter()
            .filter(|m| m.target == Game::BGEE)
            .collect();
        let second_game_mods: Vec<&GalleryMod> = entry
            .mods
            .iter()
            .filter(|m| m.target == Game::BG2EE)
            .collect();

        let bgee_keys: Vec<(&str, &str)> = first_game_mods
            .iter()
            .map(|m| (m.mod_name, m.component_id))
            .collect();
        assert_eq!(
            bgee_keys,
            vec![("DlcMerger", "1"), ("EEFixPack", "0"), ("EEFixPack", "2")]
        );

        let first_bg2ee: Vec<(&str, &str)> = second_game_mods[..3]
            .iter()
            .map(|m| (m.mod_name, m.component_id))
            .collect();
        assert_eq!(
            first_bg2ee,
            vec![("EEFixPack", "0"), ("EEFixPack", "2"), ("EET", "0")]
        );
        let last_bg2ee = second_game_mods.last().expect("bg2ee mods non-empty");
        assert_eq!(last_bg2ee.mod_name, "EET_end");
        assert_eq!(last_bg2ee.component_id, "0");

        let eeex_ids: Vec<&str> = second_game_mods
            .iter()
            .filter(|m| m.mod_name == "EEex")
            .map(|m| m.component_id)
            .collect();
        assert_eq!(eeex_ids, vec!["0", "1", "2", "3", "4", "5", "6", "7"]);

        for mods in [&first_game_mods, &second_game_mods] {
            let mut seen_runs: Vec<(&str, &str)> = Vec::new();
            let mut previous: Option<(&str, &str)> = None;
            for gallery_mod in mods {
                let key = (gallery_mod.mod_name, gallery_mod.tp_file);
                if previous != Some(key) {
                    assert!(
                        !seen_runs.contains(&key),
                        "{key:?} appears in two separate runs"
                    );
                    seen_runs.push(key);
                }
                previous = Some(key);
            }
        }
    }

    #[test]
    fn stub_codes_are_byte_identical_across_calls() {
        for entry in entries() {
            let first = share_code(entry).expect("first export");
            let second = share_code(entry).expect("second export");
            assert_eq!(
                first, second,
                "{} stub code must be deterministic",
                entry.name
            );
        }
    }

    #[test]
    fn non_bg2ee_targets_land_in_the_first_game_log() {
        let entry = entries()
            .iter()
            .find(|e| e.id == "iwdee-essentials")
            .expect("Icewind Dale Essentials is in the catalog");
        let code = share_code(entry).expect("export");
        let preview = preview_modlist_share_code(&code).expect("parse");
        assert_eq!(preview.bgee_entries, 1);
        assert_eq!(preview.bg2ee_entries, 0);
    }

    fn log_lines(text: &str) -> Vec<&str> {
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect()
    }

    #[test]
    fn fixpack_lists_install_core_fixes_and_game_text_update_on_every_tab() {
        let fixes_entry = entries()
            .iter()
            .find(|e| e.id == "eet-plus-fixes")
            .expect("EET + Fixes is in the catalog");
        let fixes_code = share_code(fixes_entry).expect("export");
        let fixes_preview = preview_modlist_share_code(&fixes_code).expect("parse");
        assert_eq!(fixes_preview.bgee_entries, 3);
        assert_eq!(fixes_preview.bg2ee_entries, 4);

        let first_game_lines = log_lines(&fixes_preview.bgee_log_text);
        assert!(first_game_lines[0].contains("DLCMERGER.TP2~ #0 #1"));
        assert!(first_game_lines[1].contains("SETUP-EEFIXPACK.TP2~ #0 #0"));
        assert!(first_game_lines[2].contains("SETUP-EEFIXPACK.TP2~ #0 #2"));

        let second_game_lines = log_lines(&fixes_preview.bg2ee_log_text);
        assert!(second_game_lines[0].contains("SETUP-EEFIXPACK.TP2~ #0 #0"));
        assert!(second_game_lines[1].contains("SETUP-EEFIXPACK.TP2~ #0 #2"));
        assert!(second_game_lines[2].contains("EET.TP2~ #0 #0"));
        assert!(second_game_lines[3].contains("EET_END.TP2~ #0 #0"));

        let vanilla_entry = entries()
            .iter()
            .find(|e| e.id == "bgee-vanilla-plus")
            .expect("BGEE Vanilla+ (with DLC) is in the catalog");
        let vanilla_code = share_code(vanilla_entry).expect("export");
        let vanilla_preview = preview_modlist_share_code(&vanilla_code).expect("parse");
        assert_eq!(vanilla_preview.bgee_entries, 4);

        let vanilla_lines = log_lines(&vanilla_preview.bgee_log_text);
        assert!(vanilla_lines[0].contains("DLCMERGER.TP2~ #0 #1"));
        assert!(vanilla_lines[1].contains("SETUP-EEFIXPACK.TP2~ #0 #0"));
        assert!(vanilla_lines[2].contains("SETUP-EEFIXPACK.TP2~ #0 #2"));
        assert!(vanilla_lines[3].contains("SETUP-CDTWEAKS.TP2~ #0 #2010"));

        let no_dlc_entry = entries()
            .iter()
            .find(|e| e.id == "bgee-vanilla-plus-no-dlc")
            .expect("BGEE Vanilla+ (no DLC) is in the catalog");
        let no_dlc_code = share_code(no_dlc_entry).expect("export");
        let no_dlc_preview = preview_modlist_share_code(&no_dlc_code).expect("parse");
        assert_eq!(no_dlc_preview.bgee_entries, 2);

        let no_dlc_lines = log_lines(&no_dlc_preview.bgee_log_text);
        assert!(no_dlc_lines[0].contains("SETUP-EEFIXPACK.TP2~ #0 #0"));
        assert!(no_dlc_lines[1].contains("SETUP-EEFIXPACK.TP2~ #0 #2"));
        assert!(!no_dlc_preview.bgee_log_text.contains("DLCMERGER.TP2"));
        assert!(!no_dlc_preview.bgee_log_text.contains("CDTWEAKS"));
    }

    #[test]
    fn eet_lists_carry_the_bg1_folder_prompt_on_the_eet_core_line() {
        for id in ["eet-essentials", "eet-plus-fixes"] {
            let entry = entries()
                .iter()
                .find(|e| e.id == id)
                .expect("EET entry is in the catalog");
            let code = share_code(entry).expect("export");
            let preview = preview_modlist_share_code(&code).expect("parse");
            let bg2ee_lines = log_lines(&preview.bg2ee_log_text);
            let marker_lines: Vec<&&str> = bg2ee_lines
                .iter()
                .filter(|line| line.contains("@wlb-inputs:"))
                .collect();
            assert_eq!(
                marker_lines.len(),
                1,
                "{} must carry exactly one prompt marker line",
                entry.name
            );
            let marker_line = marker_lines[0];
            assert!(marker_line.contains("EET.TP2~ #0 #0"));
            assert!(
                marker_line.ends_with(r"// @wlb-inputs: y,C:\BIO\Baldur's Gate Enhanced Edition")
            );
        }
    }

    #[test]
    fn only_eet_core_lines_carry_a_prompt_marker() {
        let total: usize = entries()
            .iter()
            .map(|entry| {
                let code = share_code(entry).expect("export");
                let preview = preview_modlist_share_code(&code).expect("parse");
                let first_game_hits = log_lines(&preview.bgee_log_text)
                    .iter()
                    .filter(|line| line.contains("@wlb-inputs:"))
                    .count();
                let second_game_hits = log_lines(&preview.bg2ee_log_text)
                    .iter()
                    .filter(|line| line.contains("@wlb-inputs:"))
                    .count();
                first_game_hits + second_game_hits
            })
            .sum();
        assert_eq!(total, 2);
    }

    fn parsed_lines(text: &str) -> Vec<crate::mods::component::Component> {
        log_lines(text)
            .into_iter()
            .map(|line| {
                crate::mods::component::Component::parse_weidu_line(line)
                    .expect("payload line parses")
            })
            .collect()
    }

    #[test]
    fn payload_lines_match_the_card_mod_list() {
        for entry in entries() {
            let code = share_code(entry).expect("export");
            let preview = preview_modlist_share_code(&code).expect("parse");

            let first_game_components = parsed_lines(&preview.bgee_log_text);
            let second_game_components = parsed_lines(&preview.bg2ee_log_text);

            let first_game_mods: Vec<&GalleryMod> = entry
                .mods
                .iter()
                .filter(|m| m.target != Game::BG2EE)
                .collect();
            let second_game_mods: Vec<&GalleryMod> = entry
                .mods
                .iter()
                .filter(|m| m.target == Game::BG2EE)
                .collect();

            assert_eq!(
                first_game_components.len(),
                first_game_mods.len(),
                "{} bgee tab count mismatch",
                entry.name
            );
            assert_eq!(
                second_game_components.len(),
                second_game_mods.len(),
                "{} bg2ee tab count mismatch",
                entry.name
            );

            for (component, gallery_mod) in first_game_components
                .iter()
                .zip(first_game_mods.iter())
                .chain(second_game_components.iter().zip(second_game_mods.iter()))
            {
                assert!(
                    component.name.eq_ignore_ascii_case(gallery_mod.mod_name),
                    "{} folder mismatch for {}",
                    entry.name,
                    gallery_mod.mod_name
                );
                assert!(
                    component.tp_file.eq_ignore_ascii_case(gallery_mod.tp_file),
                    "{} tp2 mismatch for {}",
                    entry.name,
                    gallery_mod.mod_name
                );
                assert_eq!(
                    component.component, gallery_mod.component_id,
                    "{} component id mismatch for {}",
                    entry.name, gallery_mod.mod_name
                );
            }
        }
    }

    #[test]
    fn versioned_lines_keep_their_labels() {
        fn split_label(label: &str) -> (String, String) {
            let mut parts = label.splitn(2, "->");
            let name = parts.next().unwrap_or_default().trim().to_string();
            let sub = parts.next().unwrap_or_default().trim().to_string();
            (name, sub)
        }

        for entry in entries() {
            let code = share_code(entry).expect("export");
            let preview = preview_modlist_share_code(&code).expect("parse");

            let first_game_components = parsed_lines(&preview.bgee_log_text);
            let second_game_components = parsed_lines(&preview.bg2ee_log_text);

            let first_game_mods: Vec<&GalleryMod> = entry
                .mods
                .iter()
                .filter(|m| m.target != Game::BG2EE)
                .collect();
            let second_game_mods: Vec<&GalleryMod> = entry
                .mods
                .iter()
                .filter(|m| m.target == Game::BG2EE)
                .collect();

            for (component, gallery_mod) in first_game_components
                .iter()
                .zip(first_game_mods.iter())
                .chain(second_game_components.iter().zip(second_game_mods.iter()))
            {
                if component.version.is_empty() {
                    continue;
                }
                let (expected_name, expected_sub) = split_label(gallery_mod.component_label);
                assert_eq!(
                    component.component_name, expected_name,
                    "{} {} component_name mismatch",
                    entry.name, gallery_mod.mod_name
                );
                assert_eq!(
                    component.sub_component, expected_sub,
                    "{} {} sub_component mismatch",
                    entry.name, gallery_mod.mod_name
                );
                assert!(
                    !component.version.is_empty(),
                    "{} {} must carry a version",
                    entry.name,
                    gallery_mod.mod_name
                );
            }
        }
    }

    #[test]
    fn eet_essentials_shows_real_versions_in_the_inside_model() {
        use crate::app::mod_downloads::source_tiers_from_texts;
        use crate::ui::install::inside_model;

        let entry = entries()
            .iter()
            .find(|e| e.id == "eet-essentials")
            .expect("EET Essentials is in the catalog");
        let code = share_code(entry).expect("export");
        let preview = preview_modlist_share_code(&code).expect("parse");
        let tiers = source_tiers_from_texts(
            include_str!("../../../core/config/default_mod_downloads.toml"),
            "",
            &preview.source_overrides_text,
        );
        let model = inside_model::build(&preview, &tiers);

        let version_of = |folder: &str| -> String {
            model
                .sections
                .iter()
                .find_map(|section| {
                    section
                        .mods
                        .iter()
                        .find(|group| group.folder.eq_ignore_ascii_case(folder))
                        .map(|group| group.version.clone())
                })
                .unwrap_or_else(|| panic!("{folder} must appear in the inside model"))
        };

        assert_eq!(version_of("DlcMerger"), "1.8");
        assert_eq!(version_of("EEFixPack"), "");
        assert_eq!(version_of("EET"), "v14.0");
        assert_eq!(version_of("CDTweaks"), "v18");
        assert_eq!(version_of("LeUI"), "4.9.1");
    }
}
