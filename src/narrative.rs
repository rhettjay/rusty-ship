use crate::menu::{NarrativeProgress, DialogueContext, DialogueCallback};
use crate::boss::BossType;
use crate::config::CONFIG;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NarrativeTrigger {
    GameStart,
    WaveCleared(u32),
    BossDefeated(BossType),
    ScoreThreshold(i32),
    AllBossesDefeated,
}

pub fn check_triggers(progress: &mut NarrativeProgress, trigger: NarrativeTrigger) -> Option<DialogueContext> {
    match trigger {
        NarrativeTrigger::GameStart => {
            if progress.current_chapter == 0 && progress.flags.is_empty() {
                Some(intro_dialogue())
            } else {
                None
            }
        }
        NarrativeTrigger::WaveCleared(wave) => {
            progress.current_wave = wave;
            check_boss_trigger(wave, progress)
        }
        NarrativeTrigger::BossDefeated(boss_type) => {
            progress.defeated_bosses.insert(boss_type);
            progress.current_chapter += 1;
            Some(boss_defeat_dialogue(boss_type))
        }
        NarrativeTrigger::ScoreThreshold(score) => {
            if score >= CONFIG.bonus_score_threshold 
                && !progress.defeated_bosses.contains(&BossType::Deadbeef)
                && !progress.flags.contains("deadbeef_spawned") {
                progress.flags.insert("deadbeef_spawned".to_string());
                Some(bonus_boss_dialogue())
            } else {
                None
            }
        }
        NarrativeTrigger::AllBossesDefeated => {
            if progress.defeated_bosses.len() >= 5 {
                Some(victory_dialogue())
            } else {
                None
            }
        }
    }
}

fn check_boss_trigger(wave: u32, progress: &NarrativeProgress) -> Option<DialogueContext> {
    let boss_waves = CONFIG.boss_waves.clone();
    if let Some(idx) = boss_waves.iter().position(|&w| w == wave) {
        if idx < 5 && !progress.defeated_bosses.contains(&BOSS_ORDER[idx]) {
            return Some(boss_intro_dialogue(BOSS_ORDER[idx]));
        }
    }
    None
}

const BOSS_ORDER: [BossType; 5] = [
    BossType::Blowfish,
    BossType::Twofish,
    BossType::RufusReverse,
    BossType::MollyHashpass,
    BossType::CaptainDavey,
];

fn intro_dialogue() -> DialogueContext {
    DialogueContext {
        dialogue_id: "intro".to_string(),
        on_complete: DialogueCallback::ResumeGame,
    }
}

fn boss_intro_dialogue(boss: BossType) -> DialogueContext {
    DialogueContext {
        dialogue_id: format!("boss_intro_{:?}", boss).to_lowercase(),
        on_complete: DialogueCallback::SpawnBoss(boss),
    }
}

fn boss_defeat_dialogue(boss: BossType) -> DialogueContext {
    DialogueContext {
        dialogue_id: format!("boss_defeat_{:?}", boss).to_lowercase(),
        on_complete: DialogueCallback::NextChapter,
    }
}

fn bonus_boss_dialogue() -> DialogueContext {
    DialogueContext {
        dialogue_id: "bonus_boss_deadbeef".to_string(),
        on_complete: DialogueCallback::SpawnBoss(BossType::Deadbeef),
    }
}

fn victory_dialogue() -> DialogueContext {
    DialogueContext {
        dialogue_id: "victory".to_string(),
        on_complete: DialogueCallback::GameComplete,
    }
}

pub fn get_chapter_name(chapter: u8) -> &'static str {
    match chapter {
        0 => "Prologue: Portscan Protocol",
        1 => "Chapter 1: Blowfish's Burst",
        2 => "Chapter 2: Twofish's Twin Terrors",
        3 => "Chapter 3: Rufus Reverse's Ruse",
        4 => "Chapter 4: Molly Hashpass's Healing",
        5 => "Chapter 5: Captain Davey's Portscan",
        6 => "Epilogue: Deadbeef's Kitchen",
        _ => "Unknown Chapter",
    }
}