"use strict";

const $ = (selector) => document.querySelector(selector);
const fragment = location.hash.slice(1);
let token = fragment || sessionStorage.getItem("klm-session");
if (fragment) {
  sessionStorage.setItem("klm-session", fragment);
  history.replaceState(null, "", location.pathname);
}

const notice = $("#notice");
function showError(message, disconnected = false) {
  notice.textContent = message;
  notice.hidden = false;
  if (disconnected) $("#connection").lastChild.textContent = " Déconnecté";
}
function clearError() {
  notice.hidden = true;
  $("#connection").lastChild.textContent = " Connecté";
}
function element(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = String(text);
  if (className) node.className = className;
  return node;
}
async function api(path, options = {}) {
  if (!token) throw new Error("Ouvrez le lien complet affiché par « klm ui » pour accéder à la session.");
  const response = await fetch(path, {
    ...options,
    headers: { Authorization: `Bearer ${token}`, ...(options.headers || {}) },
    cache: "no-store",
  });
  const result = await response.json();
  if (!response.ok) throw new Error(result.error || `Erreur HTTP ${response.status}`);
  return result;
}
let timerState = null;
let timerRefreshRunning = false;
let nextTimerRetry = 0;
const backupWarning = "Fermez KLM avant la sauvegarde : le registre est verrouillé.";
function updateTimers() {
  let due = false;
  for (const [key, selector] of [["research", "#research-timer"], ["backup", "#backup-timer"]]) {
    const node = $(selector);
    if (!timerState) {
      node.textContent = "Indisponible";
      continue;
    }
    $("#home-next").textContent = $("#research-timer").textContent;
    const timer = timerState[key];
    if (timer.status === "missing") {
      node.textContent = "Minuteur introuvable";
    } else if (timer.status === "unscheduled") {
      node.textContent = "Non programmé";
    } else {
      const seconds = Math.ceil((timer.next_ms - Date.now()) / 1000);
      if (seconds <= 0) {
        node.textContent = "Échéance atteinte…";
        due = true;
      } else if (key === "research") {
        node.textContent = `Dans ${Math.floor(seconds / 60)} min ${String(seconds % 60).padStart(2, "0")} s`;
      } else {
        const minutes = Math.ceil(seconds / 60);
        const hours = Math.floor(minutes / 60);
        node.textContent = hours ? `Dans ${hours} h ${String(minutes % 60).padStart(2, "0")} min` : `Dans ${minutes} min`;
      }
    }
  }
  if (due && Date.now() >= nextTimerRetry) {
    nextTimerRetry = Date.now() + 5000;
    refreshTimers();
  }
}
async function refreshTimers() {
  if (timerRefreshRunning) return;
  timerRefreshRunning = true;
  try {
    const timers = await api("/api/timers");
    for (const timer of [timers.research, timers.backup]) {
      if (!timer || !["scheduled", "unscheduled", "missing"].includes(timer.status)
          || (timer.status === "scheduled" && !Number.isSafeInteger(timer.next_ms))) {
        throw new Error("Données des minuteurs invalides.");
      }
    }
    timerState = timers;
    $("#timer-note").textContent = backupWarning;
  } catch (error) {
    timerState = null;
    $("#timer-note").textContent = `Minuteurs indisponibles : ${error.message} ${backupWarning}`;
  } finally {
    timerRefreshRunning = false;
  }
  updateTimers();
}
function stat(label, value) {
  const node = element("div", undefined, "stat");
  node.append(element("span", label), element("strong", value));
  return node;
}
function render(state) {
  $("#nav-hypotheses").textContent = state.hypotheses_count;
  $("#nav-reflections").textContent = state.reflections_count;
  $("#status-summary").replaceChildren(
    stat("Sanctuaire", state.sanctuary),
    stat("Époque", state.epoch),
    stat("Questions en attente", state.questions.length)
  );
  const brains = state.brains.map((brain) => {
    const node = element("article", undefined, "brain");
    node.append(element("strong", `Cerveau ${brain.id}`), element("p", `${brain.concepts} concepts`),
      element("p", `Tension : ${Number(brain.tension).toFixed(2)}`), element("span", brain.phase, "phase"));
    return node;
  });
  $("#brains").replaceChildren(...brains);
  const night = state.night_review;
  $("#night-summary").replaceChildren(...(night ? [
    stat("Concepts examinés", night.concept_examinations),
    stat("Hypothèses examinées", night.hypothesis_examinations),
    stat("Dernier passage", night.last_run ? new Date(night.last_run * 1000).toLocaleString("fr-FR") : "—")
  ] : []));
}

function hypothesisNode(result) {
    const item = result.hypothesis;
    const node = element("article", undefined, "entry");
    node.append(element("span", `HYPOTHÈSE #${item.id} · NON VÉRIFIÉE · RÉVISION ${item.corrections.length}`, "entry-meta"),
      element("h3", item.statement));
    if (result.examinations !== null) {
      node.append(element("p", `${result.examinations} examen(s) nocturne(s) · Dernier : ${new Date(result.examined_at * 1000).toLocaleString("fr-FR")}`));
    }
    const sources = item.evidence.filter((entry) => entry.revision === item.corrections.length);
    if (!sources.length) node.append(element("p", "Aucune source pour cette révision."));
    else {
      const list = element("ul");
      for (const source of sources) {
        const row = element("li");
        row.append(document.createTextNode(`${source.relation} · `));
        try {
          const url = new URL(source.source);
          if (url.protocol === "https:" || url.protocol === "http:") {
            const link = element("a", source.source);
            link.href = url.href;
            link.target = "_blank";
            link.rel = "noopener noreferrer";
            row.append(link);
          } else row.append(document.createTextNode(source.source));
        } catch {
          row.append(document.createTextNode(source.source));
        }
        list.append(row);
      }
      node.append(list);
    }
    return node;
}
function reflectionNode(item) {
    const node = element("article", undefined, "entry");
    const date = new Date(item.created_at * 1000).toLocaleString("fr-FR");
    node.append(element("span", `${date} · CERVEAU ${item.brain} · ${item.stage}`, "entry-meta"),
      element("h3", item.maieutic_question), element("p", item.interior_monologue));
    return node;
}
function nightNode(item) {
    if (item.kind === "Hypothesis") {
      const review = item.value;
      const node = element("article", undefined, "entry");
      node.append(element("span", `HYPOTHÈSE #${review.id} · RÉVISION ${review.revision} · ${review.examinations} EXAMEN(S) · NON VÉRIFIÉE`, "entry-meta"),
        element("h3", review.statement),
        element("p", `Sources enregistrées : ${review.reports} signalement(s), ${review.supports} soutien(s), ${review.contradicts} contradiction(s).`));
      for (const contradiction of review.contradictions) node.append(element("p", `Lien contradictoire enregistré : ${contradiction}`));
      for (const question of review.questions) node.append(element("p", question));
      if (review.wikipedia) {
        node.append(element("p", `Extrait Wikipédia consulté le ${new Date(review.wikipedia.examined_at * 1000).toLocaleString("fr-FR")} : ${review.wikipedia.excerpt}`),
          element("p", `Source : ${review.wikipedia.source} — cet extrait ne vérifie pas automatiquement l'hypothèse.`));
      }
      return node;
    }
    const concept = item.value;
    const node = element("article", undefined, "entry");
    node.append(element("span", `CERVEAU ${concept.brain} · CONCEPT`, "entry-meta"),
      element("h3", concept.label), element("p", concept.question));
    for (const contradiction of concept.contradictions) node.append(element("p", `Lien contradictoire enregistré avec : ${contradiction}`));
    return node;
}

function knowledgeNode(item) {
  const node = element("article", undefined, "entry");
  node.append(element("span", `CERVEAU ${item.brain} · CONNAISSANCE`, "entry-meta"),
    element("h3", item.label), element("p", item.description || "Aucune description."));
  if (item.source_domain) node.append(element("p", `Domaine d'origine déclaré : ${item.source_domain} (provenance non vérifiée).`));
  node.append(element("p", item.questions.length ? "Questions ouvertes liées :" : "Aucune question ouverte liée."));
  for (const question of item.questions) node.append(element("p", question));
  node.append(element("p", item.sources.length ? "Sources des hypothèses explicitement liées :" : "Aucune source associée explicitement."));
  for (const source of item.sources) {
    const row = element("p", `Hypothèse #${source.hypothesis_id} · ${source.evidence.relation} · ${source.evidence.source}`);
    node.append(row);
  }
  return node;
}

function morningNode(item) {
  const node = element("article", undefined, "entry");
  node.append(element("span", item.kind, "entry-meta"), element("h3", item.label));
  if (!item.contradictions.length) node.append(element("p", "Aucune contradiction enregistrée lors des passages."));
  for (const contradiction of item.contradictions) node.append(element("p", `Contradiction enregistrée : ${contradiction}`));
  return node;
}

function renderMorning(summary) {
  const container = $("#morning-summary");
  if (!summary) {
    container.replaceChildren(element("div", "Aucun bilan enregistré depuis l'activation de cette fonctionnalité.", "empty"));
    return;
  }
  const status = summary.ongoing ? "Période en cours" : "Période terminée";
  container.replaceChildren(
    stat("Date · période", `${summary.date} · ${status}`),
    stat("Passages enregistrés", summary.cycles),
    stat("Sujets distincts", summary.subjects),
    stat("Examens de concepts", summary.concept_examinations),
    stat("Examens d'hypothèses", summary.hypothesis_examinations),
    stat("Sujets avec contradiction", summary.flagged),
    element("p", `Dernier passage : ${new Date(summary.last_run * 1000).toLocaleString("fr-FR")}. Le bilan peut être partiel si des passages ont été manqués.`, "summary-note")
  );
}

const lists = [
  {prefix: "hyp", endpoint: "hypotheses", render: hypothesisNode, empty: "Aucune hypothèse enregistrée."},
  {prefix: "ref", endpoint: "reflections", render: reflectionNode, empty: "Aucune réflexion pendant les dernières 24 heures."},
  {prefix: "night", endpoint: "night", render: nightNode, empty: "Aucun examen nocturne enregistré."},
  {prefix: "knowledge", endpoint: "knowledge", render: knowledgeNode, empty: "Aucune connaissance enregistrée."},
  {prefix: "morning", endpoint: "morning", render: morningNode, empty: "Aucun sujet dans le dernier bilan."},
].map((list) => ({...list, page: 1, displayedPage: 1, pages: 1, sequence: 0, debounce: null}));

async function loadList(list) {
  const sequence = ++list.sequence;
  const input = $(`#${list.prefix}-search`);
  const controls = $(`#${list.prefix}-pages`);
  controls.querySelector(".previous").disabled = true;
  controls.querySelector(".next").disabled = true;
  const query = new URLSearchParams({page: list.page, q: input.value});
  let result;
  try {
    result = await api(`/api/${list.endpoint}?${query}`);
  } catch (error) {
    if (sequence !== list.sequence) return;
    list.page = list.displayedPage;
    controls.querySelector(".previous").disabled = list.page <= 1;
    controls.querySelector(".next").disabled = list.page >= list.pages;
    throw error;
  }
  if (sequence !== list.sequence) return;
  if (list.endpoint === "morning") renderMorning(result.summary);
  list.page = result.page;
  list.displayedPage = result.page;
  list.pages = result.pages;
  const nodes = result.items.map(list.render);
  const empty = input.value.trim() ? "Aucun résultat pour cette recherche." : list.empty;
  $(`#${list.prefix}-list`).replaceChildren(...(nodes.length ? nodes : [element("div", empty, "empty")]));
  controls.querySelector("span").textContent = `${result.total} résultat(s) · Page ${result.page} sur ${result.pages}`;
  controls.querySelector(".previous").disabled = result.page <= 1;
  controls.querySelector(".next").disabled = result.page >= result.pages;
}

let activityRefreshRunning = false;
function serviceState(service) {
  if (service.active === "active") return "En cours";
  if (service.active === "inactive") return "Inactif";
  if (service.active === "failed") return "En échec";
  return `${service.active} · ${service.substate}`;
}
async function refreshActivity() {
  if (activityRefreshRunning) return;
  activityRefreshRunning = true;
  const errorNode = $("#activity-error");
  try {
    const activity = await api("/api/activity");
    const research = activity.services.find((service) => service.id === "klm-research.service");
    const backup = activity.services.find((service) => service.id === "klm-backup.service");
    if (!research || !backup || !["day", "night"].includes(activity.mode) || !Array.isArray(activity.events)) {
      throw new Error("Données d'activité invalides.");
    }
    $("#home-mode").textContent = activity.mode === "night" ? "Examen nocturne" : "Recherches de jour";
    $("#home-mode-note").textContent = activity.mode === "night"
      ? "Période prévue de minuit à 8 h, sur le dernier snapshot sauvegardé."
      : "Période prévue de 8 h à minuit. Les recherches sont importées au redémarrage de KLM.";
    $("#home-research").textContent = serviceState(research);
    $("#home-backup").textContent = serviceState(backup);
    const labels = {error: "ERREUR", warning: "AVERTISSEMENT", started: "DÉMARRAGE", finished: "TERMINÉ", info: "INFORMATION"};
    const rows = activity.events.map((event) => {
      const node = element("article", undefined, "entry");
      const service = event.unit === "klm-backup.service" ? "SAUVEGARDE" : "RECHERCHE / EXAMEN";
      node.append(element("span", `${new Date(event.timestamp_ms).toLocaleString("fr-FR")} · ${service} · ${labels[event.status] || event.status}`, "entry-meta"),
        element("p", event.message));
      return node;
    });
    $("#activity-list").replaceChildren(...(rows.length ? rows : [element("div", "Aucun événement trouvé.", "empty")]));
    errorNode.hidden = true;
  } catch (error) {
    $("#home-mode").textContent = "Indisponible";
    $("#home-mode-note").textContent = `Impossible de lire l'activité : ${error.message}`;
    $("#home-research").textContent = "Indisponible";
    $("#home-backup").textContent = "Indisponible";
    errorNode.textContent = `Journal indisponible : ${error.message}`;
    errorNode.hidden = false;
    $("#activity-list").replaceChildren();
  } finally {
    activityRefreshRunning = false;
  }
}

for (const list of lists) {
  const controls = $(`#${list.prefix}-pages`);
  controls.querySelector(".previous").disabled = true;
  controls.querySelector(".next").disabled = true;
  const reload = () => loadList(list).catch((error) => showError(error.message));
  controls.querySelector(".previous").addEventListener("click", () => { list.page--; reload(); });
  controls.querySelector(".next").addEventListener("click", () => { list.page++; reload(); });
  $(`#${list.prefix}-search`).addEventListener("input", () => {
    list.page = 1;
    list.sequence++;
    controls.querySelector(".previous").disabled = true;
    controls.querySelector(".next").disabled = true;
    clearTimeout(list.debounce);
    list.debounce = setTimeout(reload, 250);
  });
}
async function refresh() {
  try {
    render(await api("/api/state"));
    await Promise.all(lists.map(loadList));
    clearError();
  } catch (error) {
    showError(error.message, true);
  }
}
document.querySelectorAll(".nav").forEach((button) => button.addEventListener("click", () => {
  document.querySelectorAll(".nav").forEach((nav) => {
    nav.classList.toggle("active", nav === button);
    nav.removeAttribute("aria-current");
  });
  button.setAttribute("aria-current", "page");
  document.querySelectorAll(".panel").forEach((panel) => panel.classList.toggle("active", panel.id === button.dataset.panel));
  $("#top-label").textContent = button.textContent.trim().replace(/\s+\d+$/, "").toUpperCase();
}));
$("#refresh").addEventListener("click", () => { refresh(); refreshTimers(); refreshActivity(); });
$("#message").addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
    event.preventDefault();
    $("#chat-form").requestSubmit();
  }
});
$("#chat-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  const input = $("#message");
  const text = input.value.trim();
  if (!text) return;
  const send = $("#send");
  send.disabled = true;
  try {
    const result = await api("/api/chat", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ message: text }),
    });
    const messages = $("#messages");
    const welcome = messages.querySelector(".welcome");
    if (welcome) welcome.remove();
    messages.append(element("div", text, "bubble user"), element("div", result.response, "bubble assistant"));
    messages.scrollTop = messages.scrollHeight;
    input.value = "";
    await refresh();
  } catch (error) {
    showError(error.message);
  } finally {
    send.disabled = false;
    input.focus();
  }
});
refresh();
refreshTimers();
refreshActivity();
setInterval(updateTimers, 1000);
setInterval(refreshTimers, 15000);
setInterval(refreshActivity, 30000);
