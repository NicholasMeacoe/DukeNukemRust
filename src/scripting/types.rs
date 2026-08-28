#![allow(dead_code)]

pub const NUM_KEYWORDS: usize = 112;
pub const MAX_TILES: usize = 6144;
pub const TICSPERFRAME: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Opcode {
    DefineLevelName = 0,
    Actor = 1,
    AddAmmo = 2,
    IfRnd = 3,
    EndA = 4,
    IfCanSee = 5,
    IfHitWeapon = 6,
    Action = 7,
    IfPDistL = 8,
    IfPDistG = 9,
    Else = 10,
    Strength = 11,
    Break = 12,
    Shoot = 13,
    PalFrom = 14,
    Sound = 15,
    Fall = 16,
    State = 17,
    EndS = 18,
    Define = 19,
    IfAi = 21,
    Killit = 22,
    AddWeapon = 23,
    Ai = 24,
    AddPHealth = 25,
    IfDead = 26,
    IfSquished = 27,
    SizeTo = 28,
    LeftBrace = 29,
    RightBrace = 30,
    Spawn = 31,
    Move = 32,
    IfWasWeapon = 33,
    IfAction = 34,
    IfActionCount = 35,
    ResetActionCount = 36,
    Debris = 37,
    PStomp = 38,
    CStat = 40,
    IfMove = 41,
    ResetPlayer = 42,
    IfOnWater = 43,
    IfInWater = 44,
    IfCanShootTarget = 45,
    IfCount = 46,
    ResetCount = 47,
    AddInventory = 48,
    IfActorNotStayput = 49,
    HitRadius = 50,
    IfP = 51,
    Count = 52,
    IfActor = 53,
    IfStrength = 56,
    Guts = 58,
    IfSpawnedBy = 59,
    WackPlayer = 61,
    IfGapZL = 62,
    IfHitSpace = 63,
    IfOutside = 64,
    IfMultiplayer = 65,
    Operate = 66,
    IfInSpace = 67,
    Debug = 68,
    EndOfGame = 69,
    IfBulletNear = 70,
    IfRespawn = 71,
    IfFloorDistL = 72,
    IfCeilingDistL = 73,
    SpritePal = 74,
    IfPInventory = 75,
    CActor = 77,
    IfPHealthL = 78,
    Quote = 80,
    IfInOuterSpace = 81,
    IfNotMoving = 82,
    RespawnHitag = 83,
    Tip = 84,
    IfSpritePal = 85,
    Money = 86,
    SoundOnce = 87,
    AddKills = 88,
    StopSound = 89,
    IfAwayFromWall = 90,
    IfCanSeeTarget = 91,
    GlobalSound = 92,
    LotsOfGlass = 93,
    IfGotWeaponCe = 94,
    GetLastPal = 95,
    PKick = 96,
    MikeSnd = 97,
    UserActor = 98,
    SizeAt = 99,
    AddStrength = 100,
    CStatOr = 101,
    Mail = 102,
    Paper = 103,
    TossWeapon = 104,
    SleepTime = 105,
    NullOp = 106,
    IfNoSounds = 109,
    ClipDist = 110,
    IfAngDiffL = 111,
    Unknown = -1,
}

impl Opcode {
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

impl From<Opcode> for i32 {
    fn from(op: Opcode) -> Self {
        op as i32
    }
}

impl From<i32> for Opcode {
    fn from(val: i32) -> Self {
        match val {
            0 => Opcode::DefineLevelName,
            1 => Opcode::Actor,
            2 => Opcode::AddAmmo,
            3 => Opcode::IfRnd,
            4 => Opcode::EndA,
            5 => Opcode::IfCanSee,
            6 => Opcode::IfHitWeapon,
            7 => Opcode::Action,
            8 => Opcode::IfPDistL,
            9 => Opcode::IfPDistG,
            10 => Opcode::Else,
            11 => Opcode::Strength,
            12 => Opcode::Break,
            13 => Opcode::Shoot,
            14 => Opcode::PalFrom,
            15 => Opcode::Sound,
            16 => Opcode::Fall,
            17 => Opcode::State,
            18 => Opcode::EndS,
            19 => Opcode::Define,
            21 => Opcode::IfAi,
            22 => Opcode::Killit,
            23 => Opcode::AddWeapon,
            24 => Opcode::Ai,
            25 => Opcode::AddPHealth,
            26 => Opcode::IfDead,
            27 => Opcode::IfSquished,
            28 => Opcode::SizeTo,
            29 => Opcode::LeftBrace,
            30 => Opcode::RightBrace,
            31 => Opcode::Spawn,
            32 => Opcode::Move,
            33 => Opcode::IfWasWeapon,
            34 => Opcode::IfAction,
            35 => Opcode::IfActionCount,
            36 => Opcode::ResetActionCount,
            37 => Opcode::Debris,
            38 => Opcode::PStomp,
            40 => Opcode::CStat,
            41 => Opcode::IfMove,
            42 => Opcode::ResetPlayer,
            43 => Opcode::IfOnWater,
            44 => Opcode::IfInWater,
            45 => Opcode::IfCanShootTarget,
            46 => Opcode::IfCount,
            47 => Opcode::ResetCount,
            48 => Opcode::AddInventory,
            49 => Opcode::IfActorNotStayput,
            50 => Opcode::HitRadius,
            51 => Opcode::IfP,
            52 => Opcode::Count,
            53 => Opcode::IfActor,
            56 => Opcode::IfStrength,
            58 => Opcode::Guts,
            59 => Opcode::IfSpawnedBy,
            61 => Opcode::WackPlayer,
            62 => Opcode::IfGapZL,
            63 => Opcode::IfHitSpace,
            64 => Opcode::IfOutside,
            65 => Opcode::IfMultiplayer,
            66 => Opcode::Operate,
            67 => Opcode::IfInSpace,
            68 => Opcode::Debug,
            69 => Opcode::EndOfGame,
            70 => Opcode::IfBulletNear,
            71 => Opcode::IfRespawn,
            72 => Opcode::IfFloorDistL,
            73 => Opcode::IfCeilingDistL,
            74 => Opcode::SpritePal,
            75 => Opcode::IfPInventory,
            77 => Opcode::CActor,
            78 => Opcode::IfPHealthL,
            80 => Opcode::Quote,
            81 => Opcode::IfInOuterSpace,
            82 => Opcode::IfNotMoving,
            83 => Opcode::RespawnHitag,
            84 => Opcode::Tip,
            85 => Opcode::IfSpritePal,
            86 => Opcode::Money,
            87 => Opcode::SoundOnce,
            88 => Opcode::AddKills,
            89 => Opcode::StopSound,
            90 => Opcode::IfAwayFromWall,
            91 => Opcode::IfCanSeeTarget,
            92 => Opcode::GlobalSound,
            93 => Opcode::LotsOfGlass,
            94 => Opcode::IfGotWeaponCe,
            95 => Opcode::GetLastPal,
            96 => Opcode::PKick,
            97 => Opcode::MikeSnd,
            98 => Opcode::UserActor,
            99 => Opcode::SizeAt,
            100 => Opcode::AddStrength,
            101 => Opcode::CStatOr,
            102 => Opcode::Mail,
            103 => Opcode::Paper,
            104 => Opcode::TossWeapon,
            105 => Opcode::SleepTime,
            106 => Opcode::NullOp,
            109 => Opcode::IfNoSounds,
            110 => Opcode::ClipDist,
            111 => Opcode::IfAngDiffL,
            _ => Opcode::Unknown,
        }
    }
}

pub mod move_flags {
    pub const FACE_PLAYER: i32       = 1;
    pub const FACE_PLAYER_SLOW: i32  = 2;
    pub const SPIN: i32              = 32;
    pub const FACE_PLAYER_SMART: i32 = 64;
    pub const FLEE_ENEMY: i32        = 128;
    pub const JUMP_TO_PLAYER: i32    = 257;
    pub const SEEK_PLAYER: i32       = 512;
    pub const FURTHEST_DIR: i32      = 1024;
    pub const DODGE_BULLET: i32      = 4096;
    pub const GET_H: i32             = 8192;
    pub const GET_V: i32             = 16384;
    pub const RANDOM_ANGLE: i32      = 32768;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionDef {
    pub start_frame: i32,
    pub num_frames: i32,
    pub view_type: i32, // 1 = 1-view, 5 = 5-view rotation
    pub inc_val: i32,
    pub delay: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MoveDef {
    pub hvel: i32,
    pub vvel: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AiDef {
    pub action_ptr: usize,
    pub move_ptr: usize,
    pub flags: i32,
}

pub const NULL_PTR: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct ActorRegisters {
    pub count: i32,                  // T1: temp_data[0]
    pub move_ptr: Option<usize>,     // T2: temp_data[1]
    pub action_count: i32,           // T3: temp_data[2]
    pub frame_offset: i32,           // T4: temp_data[3]
    pub action_ptr: Option<usize>,   // T5: temp_data[4]
    pub ai_ptr: Option<usize>,       // T6: temp_data[5]
    pub action_delay_timer: i16,     // Internal frame animation accumulator
    pub time_to_sleep: i16,
    pub mov_flag: i16,
    pub floor_z: i32,
    pub ceiling_z: i32,
    pub last_vx: i32,
    pub last_vy: i32,
    pub actor_stay_put: i16,
    pub temp_ang: i16,
}

