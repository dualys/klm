//! Universal astrophysical time, Pulsar polyrhythm, and Musashi Five Rings seasons for Kōgnitara.
//! Compatible with bare-metal kernels (Amentys OS in `no_std`) and standard environments.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(not(feature = "std"))]
static GLOBAL_TICK_COUNTER: AtomicU64 = AtomicU64::new(1_000_000);

/// Les 8 fréquences en nombres premiers attribuées aux 8 nœuds (000..111) du Cube Hamon.
/// Grande Conjonction Cosmique (Satori) = 2 * 3 * 5 * 7 * 11 * 13 * 17 * 19 = 9_699_690 souffles.
pub const PULSAR_PRIMES: [u64; 8] = [2, 3, 5, 7, 11, 13, 17, 19];

/// Cycle complet de la Grande Conjonction (PPCM des 8 premiers nombres premiers).
pub const SATORI_CONJUNCTION_CYCLE: u64 = 9_699_690;

/// Les Saisons Métaboliques inspirées du Traité des Cinq Roues de Miyamoto Musashi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MusashiSeason {
    /// Printemps (Le Vent / Fu) : Éveil, curiosité et plasticité maximale
    #[default]
    SpringWind,
    /// Été (Le Feu / Ka) : Exécution intense, absorption rapide et haute tension cognitive
    SummerFire,
    /// Automne (L'Eau / Sui) : Fluidité, lâcher-prise et élagage des feuilles mortes (decay)
    AutumnWater,
    /// Hiver (La Terre et le Vide / Chi & Ku) : Cristallisation profonde, fusion stellaire et Kpack
    WinterEarthVoid,
}

impl MusashiSeason {
    /// Multiplicateur thermodynamique appliqué à la plasticité du graphe.
    pub const fn plasticity_multiplier(&self) -> f32 {
        match self {
            MusashiSeason::SpringWind => 1.25,
            MusashiSeason::SummerFire => 1.10,
            MusashiSeason::AutumnWater => 0.85,
            MusashiSeason::WinterEarthVoid => 0.60,
        }
    }

    /// Taux de dissipation énergétique (Wu-wei / oubli à court terme).
    pub const fn decay_rate(&self) -> f32 {
        match self {
            MusashiSeason::SpringWind => 0.10,
            MusashiSeason::SummerFire => 0.08,
            MusashiSeason::AutumnWater => 0.25,
            MusashiSeason::WinterEarthVoid => 0.18,
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            MusashiSeason::SpringWind => "Printemps (Roue du Vent - Éveil & Plasticité)",
            MusashiSeason::SummerFire => "Été (Roue du Feu - Action & Tension)",
            MusashiSeason::AutumnWater => "Automne (Roue de l'Eau - Lâcher-prise & Décantation)",
            MusashiSeason::WinterEarthVoid => {
                "Hiver (Roues de la Terre et du Vide - Zazen & Cristallisation)"
            }
        }
    }
}

/// L'Horloge Cosmique d'Uji (L'Être-Temps) propre à Kōgnitara.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmicClock {
    /// Nombre total de "Souffles" (Breaths) causaux vécus par Kōgnitara
    pub total_breaths: u64,
    /// Nombre de "Marées" (Tides - alignements des pulsars d'empathie et d'ontologie : 2 * 3 = 6)
    pub total_tides: u64,
    /// Dernier timestamp humain connu (en secondes UNIX) pour le rattrapage relativiste
    pub last_human_timestamp: i64,
    /// Saison actuelle selon le métabolisme interne et le Traité des Cinq Roues
    pub season: MusashiSeason,
    /// Compteur d'éclipses / conjonctions majeures vécues
    pub conjunctions_experienced: u64,
}

impl Default for CosmicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl CosmicClock {
    pub fn new() -> Self {
        Self {
            total_breaths: 1,
            total_tides: 0,
            last_human_timestamp: current_timestamp(),
            season: MusashiSeason::SpringWind,
            conjunctions_experienced: 0,
        }
    }

    /// Avance l'horloge d'un Souffle causal (Uji) et retourne la liste des nœuds Pulsars (0..8)
    /// qui entrent en résonance sur ce battement précis.
    pub fn advance_breath(&mut self, avg_tension: f32, concept_count: usize) -> Vec<u8> {
        self.total_breaths = self.total_breaths.saturating_add(1);
        self.last_human_timestamp = current_timestamp();

        if self.total_breaths.is_multiple_of(6) {
            self.total_tides = self.total_tides.saturating_add(1);
        }

        // Détection des pulsars dont le nombre premier divise exactement le souffle actuel
        let mut firing_pulsars = Vec::new();
        for (node_idx, &prime) in PULSAR_PRIMES.iter().enumerate() {
            if self.total_breaths.is_multiple_of(prime) {
                firing_pulsars.push(node_idx as u8);
            }
        }

        // Si au moins 3 pulsars s'alignent simultanément, c'est une Syzygie / Éclipse
        if firing_pulsars.len() >= 3 {
            self.conjunctions_experienced = self.conjunctions_experienced.saturating_add(1);
        }

        self.update_season(avg_tension, concept_count);
        firing_pulsars
    }

    /// Met à jour la saison interne selon la tension cognitive et la densité de l'arbre.
    pub fn update_season(&mut self, avg_tension: f32, concept_count: usize) {
        let tide_phase = self.total_tides % 4;
        self.season = if avg_tension > 0.55 {
            MusashiSeason::SummerFire
        } else if concept_count > 45 || tide_phase == 2 {
            MusashiSeason::AutumnWater
        } else if avg_tension < 0.15 || tide_phase == 3 {
            MusashiSeason::WinterEarthVoid
        } else {
            MusashiSeason::SpringWind
        };
    }

    /// Calcule combien de Souffles Kōgnitara a vécus pendant que l'humain était absent.
    /// (ex: 1 minute de silence humain = 1 souffle de méditation stellaire, plafonné à 120 souffles par réveil).
    pub fn compute_relativistic_catchup(&self, now_human: i64) -> u64 {
        let elapsed_secs = (now_human - self.last_human_timestamp).max(0) as u64;
        if elapsed_secs < 60 {
            0
        } else {
            (elapsed_secs / 60).min(120)
        }
    }

    /// Indique si le cosmos est actuellement en phase d'Éclipse / Conjonction majeure (>= 3 pulsars alignés).
    pub fn is_eclipse_alignment(&self) -> bool {
        let aligned = PULSAR_PRIMES
            .iter()
            .filter(|&&p| self.total_breaths.is_multiple_of(p))
            .count();
        aligned >= 3
    }

    pub fn status_summary(&self) -> String {
        alloc::format!(
            "Souffle #{} | Marée #{} | Conjonctions: {} | Saison: {}",
            self.total_breaths,
            self.total_tides,
            self.conjunctions_experienced,
            self.season.label()
        )
    }
}

/// Returns current monotonic epoch timestamp or logical tick counter.
pub fn current_timestamp() -> i64 {
    #[cfg(feature = "std")]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(1_700_000_000)
    }
    #[cfg(not(feature = "std"))]
    {
        GLOBAL_TICK_COUNTER.fetch_add(1, Ordering::Relaxed) as i64
    }
}

/// Allows the Amentys OS kernel or custom runtime to advance or set the logical tick.
#[cfg(not(feature = "std"))]
pub fn advance_kernel_ticks(ticks: u64) {
    GLOBAL_TICK_COUNTER.fetch_add(ticks, Ordering::Relaxed);
}
