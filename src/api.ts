import { invoke } from "@tauri-apps/api/core";

export interface RunDefaults {
  audio: boolean;
  language: string;
  output_codec: "h264" | "h265";
  show_skeletons: boolean;
  show_player_trajectories: boolean;
  show_court_trajectory: boolean;
  show_shuttlecock_trajectory: boolean;
  show_player_stats: boolean;
  show_pose_roi: boolean;
}
export interface Config {
  gb_cpp_path: string;
  data_dir: string;
  ball_model: string;
  pose_model: string;
  defaults: RunDefaults;
}
export interface RunParams {
  video: string;
  template: string;
  annotations: string | null;
  out_dir: string;
  audio: boolean;
  language: "en" | "zh" | "id";
  output_codec: "h264" | "h265";
  show_skeletons: boolean;
  show_player_trajectories: boolean;
  show_court_trajectory: boolean;
  show_shuttlecock_trajectory: boolean;
  show_player_stats: boolean;
  show_pose_roi: boolean;
}

export const getDefaults = () =>
  invoke<[Config | null, string, boolean]>("get_defaults");
export const startRun = (params: RunParams) => invoke<void>("start_run", { params });
export const cancelRun = () => invoke<void>("cancel_run");
export const saveConfig = (cfg: Config) => invoke<void>("save_config", { cfg });
export const saveAnnotations = (path: string, corners: number[][], mid: number) =>
  invoke<void>("save_annotations", { path, corners, midHeight: mid });
