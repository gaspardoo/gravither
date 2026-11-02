//! Minimal startup coordinator for Gravither.

/// The logical PID reserved for Sysruning in the anchor space.
pub const PID: u64 = 0;

/// A program that Sysruning knows how to schedule in the prototype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Program {
    Admin,
    Session,
    Unicrypt,
    GetIn,
    SignUp,
    Viewer,
    User,
}

/// The condition under which a program belongs in the startup plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchPhase {
    /// Programs available during system startup, before user authentication.
    SystemStartup,
    /// Programs that must wait until the user's Session and its viewer are ready.
    SessionReady,
}

/// One planned program launch. This describes ordering; it does not execute programs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchStep {
    pub program: Program,
    pub phase: LaunchPhase,
}

/// Sysruning's minimal identity and startup plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sysruning;

impl Sysruning {
    /// Sysruning's logical PID in the anchor space.
    pub fn pid(&self) -> u64 {
        PID
    }

    /// The process name.
    pub fn process_name(&self) -> &'static str {
        "sysruning"
    }

    /// Return the prototype launch order without starting the programs.
    ///
    /// Session-specific User execution is deliberately delayed until the
    /// Session and viewer are ready. Life will later coordinate that transition.
    pub fn launch_plan(&self) -> &'static [LaunchStep] {
        const PLAN: &[LaunchStep] = &[
            LaunchStep {
                program: Program::Admin,
                phase: LaunchPhase::SystemStartup,
            },
            LaunchStep {
                program: Program::Session,
                phase: LaunchPhase::SystemStartup,
            },
            LaunchStep {
                program: Program::Unicrypt,
                phase: LaunchPhase::SystemStartup,
            },
            LaunchStep {
                program: Program::GetIn,
                phase: LaunchPhase::SystemStartup,
            },
            LaunchStep {
                program: Program::SignUp,
                phase: LaunchPhase::SystemStartup,
            },
            LaunchStep {
                program: Program::Viewer,
                phase: LaunchPhase::SessionReady,
            },
            LaunchStep {
                program: Program::User,
                phase: LaunchPhase::SessionReady,
            },
        ];

        PLAN
    }
}

#[cfg(test)]
mod tests {
    use super::{LaunchPhase, Program, Sysruning, PID};

    #[test]
    fn sysruning_uses_anchor_pid_zero() {
        let sysruning = Sysruning;
        assert_eq!(sysruning.pid(), 0);
        assert_eq!(sysruning.pid(), PID);
        assert_eq!(sysruning.process_name(), "sysruning");
    }

    #[test]
    fn launch_plan_delays_viewer_and_user_until_session_is_ready() {
        let plan = Sysruning.launch_plan();

        assert_eq!(plan[0].program, Program::Admin);
        assert_eq!(plan[1].program, Program::Session);
        assert!(plan[..5]
            .iter()
            .all(|step| step.phase == LaunchPhase::SystemStartup));
        assert_eq!(plan[5].program, Program::Viewer);
        assert_eq!(plan[6].program, Program::User);
        assert!(plan[5..]
            .iter()
            .all(|step| step.phase == LaunchPhase::SessionReady));
    }
}
