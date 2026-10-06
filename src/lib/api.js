import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

export async function pickFolder() {
  const selected = await open({ directory: true, multiple: false });
  if (!selected) return null;
  const isRepo = await invoke("validate_git_repo", { path: selected });
  if (!isRepo) throw new Error("Selected folder is not a git repository");
  await invoke("start_watcher", { path: selected });
  return selected;
}

export async function getDiff(repoPath, baseBranch = null) {
  return invoke("get_diff", { repoPath, baseBranch });
}

export async function getBranchComparisonInfo(repoPath, baseBranch) {
  return invoke("get_branch_comparison_info", { repoPath, baseBranch });
}

export async function acceptHunk(repoPath, hunkId) {
  return invoke("accept_hunk", { repoPath, hunkId });
}

export async function rejectHunk(repoPath, hunkId) {
  return invoke("reject_hunk", { repoPath, hunkId });
}

export async function getRepoFiles(repoPath) {
  return invoke("get_repo_files", { repoPath });
}

export async function getFileContent(repoPath, filePath) {
  return invoke("get_file_content", { repoPath, filePath });
}

export async function saveFileContent(repoPath, filePath, content) {
  return invoke("save_file_content", { repoPath, filePath, content });
}

export async function getCurrentBranch(repoPath) {
  return invoke("get_current_branch", { repoPath });
}

export async function listBranches(repoPath) {
  return invoke("list_branches", { repoPath });
}

export async function switchBranch(repoPath, branch) {
  return invoke("switch_branch", { repoPath, branch });
}

export async function pullBranch(repoPath, branch) {
  return invoke("pull_branch", { repoPath, branch });
}

export function onChangesDetected(callback) {
  return listen("changes-detected", callback);
}

export async function setWindowBlurIntensity(intensity) {
  try {
    return await invoke("set_window_blur_intensity", { intensity });
  } catch (e) {
    console.warn("Could not set blur intensity:", e);
  }
}
