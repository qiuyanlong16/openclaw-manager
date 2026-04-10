import { useState, useEffect, useCallback, useDeferredValue } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Skill {
  name: string;
  emoji: string;
  description: string;
  eligible: boolean;
  disabled: boolean;
  source: string;
  missingBins: string[];
}

// Module-level cache: survives tab switches
let cachedSkills: Skill[] | null = null;
let fetchPromise: Promise<Skill[]> | null = null;

export default function Skills() {
  const [skills, setSkills] = useState<Skill[]>(cachedSkills ?? []);
  const [filter, setFilter] = useState<"all" | "ready" | "needs-setup">("all");
  const [search, setSearch] = useState("");
  const [saving, setSaving] = useState<string | null>(null);
  const [initialLoad, setInitialLoad] = useState(cachedSkills === null);

  const deferredFilter = useDeferredValue(filter);
  const deferredSearch = useDeferredValue(search);

  const fetchSkills = useCallback(async () => {
    if (cachedSkills !== null) return;
    if (fetchPromise !== null) return;

    fetchPromise = (async () => {
      const data = await invoke<Skill[]>("list_skills");
      cachedSkills = data;
      setSkills(data);
      setInitialLoad(false);
      return data;
    })();

    await fetchPromise;
  }, []);

  useEffect(() => {
    fetchSkills();
  }, [fetchSkills]);

  async function toggleSkill(skill: Skill) {
    setSaving(skill.name);
    try {
      await invoke("set_skill_enabled", {
        name: skill.name,
        enabled: skill.disabled,
      });
      setSkills((prev) =>
        prev.map((s) =>
          s.name === skill.name ? { ...s, disabled: !s.disabled } : s
        )
      );
      if (cachedSkills) {
        cachedSkills = cachedSkills.map((s) =>
          s.name === skill.name ? { ...s, disabled: !s.disabled } : s
        );
      }
    } catch (e) {
      console.error("Failed to toggle skill:", e);
    } finally {
      setSaving(null);
    }
  }

  const filtered = skills.filter((s) => {
    if (deferredFilter === "ready" && !s.eligible) return false;
    if (deferredFilter === "needs-setup" && s.eligible) return false;
    if (deferredSearch && !s.name.toLowerCase().includes(deferredSearch.toLowerCase()) && !s.description.toLowerCase().includes(deferredSearch.toLowerCase())) return false;
    return true;
  });

  const readyCount = skills.filter((s) => s.eligible && !s.disabled).length;
  const totalCount = skills.length;

  if (initialLoad && skills.length === 0) {
    return (
      <div className="skills-loading">
        <img src="/favicon.svg" alt="" className="lobster-loader" />
        <span>加载 Skills 列表...</span>
      </div>
    );
  }

  return (
    <div>
      <div className="skills-header">
        <h2>Skills 管理</h2>
        <span className="skills-count">{readyCount}/{totalCount} 可用</span>
      </div>

      <div className="skills-toolbar">
        <input
          className="skills-search"
          type="text"
          placeholder="搜索技能..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <div className="skills-filter">
          {(["all", "ready", "needs-setup"] as const).map((f) => (
            <button
              key={f}
              className={`skills-filter-btn ${filter === f ? "active" : ""}`}
              onClick={() => setFilter(f)}
            >
              {f === "all" ? "全部" : f === "ready" ? "就绪" : "需配置"}
            </button>
          ))}
        </div>
      </div>

      <div className="skills-list">
        {filtered.map((skill) => (
          <div key={skill.name} className={`skill-card ${skill.disabled ? "disabled" : ""} ${skill.eligible ? "ready" : ""}`}>
            <div className="skill-info">
              <span className="skill-emoji">{skill.emoji}</span>
              <div className="skill-details">
                <div className="skill-name-row">
                  <span className="skill-name">{skill.name}</span>
                  <span className={`skill-badge ${skill.eligible ? "ready" : "setup"}`}>
                    {skill.disabled ? "已禁用" : skill.eligible ? "就绪" : "需配置"}
                  </span>
                  {skill.disabled && (
                    <span className="skill-badge blocked">未启用</span>
                  )}
                </div>
                <p className="skill-desc">{skill.description}</p>
                {skill.missingBins.length > 0 && (
                  <span className="skill-missing">
                    缺少: {skill.missingBins.join(", ")}
                  </span>
                )}
              </div>
            </div>
            <button
              className={`skill-toggle ${skill.disabled ? "enable" : "disable"}`}
              onClick={() => toggleSkill(skill)}
              disabled={saving === skill.name}
            >
              {saving === skill.name
                ? "..."
                : skill.disabled
                ? "启用"
                : "禁用"}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
