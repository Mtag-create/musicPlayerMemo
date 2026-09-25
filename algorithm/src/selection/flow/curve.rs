//! 理想の流れ(Ideal Curve / Energy Curve)
//!
//! Big Wave1周分(BIG_WAVE_SIZE曲)に対応する、曲の位置ごとの目標Energy値。
//! 現段階では目標値の配列を保持するだけの入れ物。
//! 目標値をどう生成するか(上げる/維持する/ピークを作る等のパターン生成)、
//! 実測Energyとのズレをどう評価するかは、スコアリングと接続する段階で実装する。

use crate::config::BIG_WAVE_SIZE;

pub struct EnergyCurve {
    /// 曲の位置ごとの目標Energy値。長さはBIG_WAVE_SIZE。
    target_values: Vec<f32>,
}

impl EnergyCurve {
    /// 全ポジションを同じ値で初期化する(仮の初期状態)。
    /// 実際のカーブ生成ロジックは未実装。
    pub fn new_flat(value: f32) -> Self {
        EnergyCurve {
            target_values: vec![value; BIG_WAVE_SIZE],
        }
    }

    pub fn target_at(&self, position: usize) -> Option<f32> {
        self.target_values.get(position).copied()
    }

    pub fn len(&self) -> usize {
        self.target_values.len()
    }
}