// pub mod custom_constraints;

// use csv::ReaderBuilder;
use crate::{
    Projekt,
    types::{ProjektId, SaveFileSchueler, SchuelerId},
};
use gloo_console::log;
use good_lp::{
    Expression, ProblemVariables, ResolutionError, Solution, default_solver, solvers::SolverModel,
    variable,
};
use std::collections::{BTreeMap, HashSet};

// #[wasm_bindgen]
// pub fn solve_test(projects: JsValue, students: JsValue, feste_zuordnung: JsValue) -> JsValue {
//     web_sys::console::log_1(&"Creating Data".into());

//     web_sys::console::log_1(&format!("Students: {students:?}").into());
//     web_sys::console::log_1(&format!("Projects: {projects:?}").into());

//     let projects: Result<HashMap<String, Project>, serde_wasm_bindgen::Error> =
//         serde_wasm_bindgen::from_value(projects);

//     let projects = match projects {
//         Ok(projects) => projects
//             .iter()
//             .map(|(project_id, project)| (str::parse(project_id).unwrap(), project.clone()))
//             .collect::<HashMap<ProjectID, Project>>(),
//         Err(e) => {
//             web_sys::console::log_1(&format!("Failed to parse projects: {e:?}").into());
//             return JsValue::from_str("Failed to parse projects");
//         }
//     };

//     let students: Result<HashMap<StudentID, Student>, serde_wasm_bindgen::Error> =
//         serde_wasm_bindgen::from_value(students);

//     let students = match students {
//         Ok(students) => students,
//         Err(e) => {
//             web_sys::console::log_1(&format!("Failed to parse students: {e:?}").into());
//             return JsValue::from_str("Failed to parse students");
//         }
//     };

//     let feste_zuordnung: Result<HashMap<StudentID, ProjectID>, serde_wasm_bindgen::Error> =
//         serde_wasm_bindgen::from_value(feste_zuordnung);
//     let feste_zuordnung = match feste_zuordnung {
//         Ok(feste_zuordnung) => feste_zuordnung,
//         Err(e) => {
//             web_sys::console::log_1(&format!("Failed to parse feste_zuordnung: {e:?}").into());
//             return JsValue::from_str("Failed to parse feste_zuordnung");
//         }
//     };

//     web_sys::console::log_1(&format!("Students: {students:?}").into());
//     web_sys::console::log_1(&format!("Projects: {projects:?}").into());
//     web_sys::console::log_1(&format!("Feste Zuordnung: {feste_zuordnung:?}").into());

//     web_sys::console::log_1(&"Data parsed!".into());

//     web_sys::console::log_1(&"Solving...".into());

//     let solution = solve_good_lp(&projects, &students, &feste_zuordnung);

//     web_sys::console::log_1(&"Solver is done!".into());

//     match solution {
//         Ok((sol, x)) => {
//             let mut result = Vec::new();
//             for (sid, (&_student_id, student)) in students.clone().iter().enumerate() {
//                 for (pid, (&_project_id, project)) in projects.clone().iter().enumerate() {
//                     if sol.value(x[sid][pid]) > 0.5 {
//                         result.push(format!("{} assigned to {}", student.name, project.name));
//                     }
//                 }
//             }
//             result.push("Solution found".to_string());
//             // result.push((0, 0));
//             // serde_wasm_bindgen::to_value(&result).unwrap()
//             serde_wasm_bindgen::to_value(&(
//                 &x.iter()
//                     .map(|y| y.iter().map(|v| sol.value(*v)).collect::<Vec<f64>>())
//                     .collect::<Vec<_>>(),
//                 students
//                     .clone()
//                     .iter()
//                     .enumerate()
//                     .map(|(sid, s)| (sid, s.0))
//                     .collect::<HashMap<_, _>>(),
//                 projects
//                     .clone()
//                     .iter()
//                     .enumerate()
//                     .map(|(pid, p)| (pid, p.0))
//                     .collect::<HashMap<_, _>>(),
//             ))
//             .unwrap()
//         }
//         Err(e) => JsValue::from_str(&format!("Error: {e}")),
//     }
// }

pub fn solve_good_lp(
    projects: &BTreeMap<ProjektId, Projekt>,
    students: &BTreeMap<SchuelerId, SaveFileSchueler>,
    feste_zuordnung: &BTreeMap<SchuelerId, ProjektId>,
) -> Result<Vec<Vec<f64>>, ResolutionError> {
    web_sys::console::log_1(&"Creating parameters".into());
    let weights = [5.0, 4.0, 3.0, 2.0, 1.0];
    let partner_weight = 2.0;

    let student_ids: Vec<SchuelerId> = students.keys().cloned().collect();
    let project_ids: Vec<ProjektId> = projects.keys().cloned().collect();

    let mut vars = ProblemVariables::new();
    let n = student_ids.len();
    let _m = project_ids.len();

    web_sys::console::log_1(&"Projektids starting from 0".into());
    // todo!("Projektids starting from 0");

    web_sys::console::log_1(&"Creating decision vars".into());

    // decision vars x[s][p]
    let mut x = vec![vec![]; n];
    for (si, &sid) in student_ids.iter().enumerate() {
        for &pid in &project_ids {
            let v = vars.add(variable().binary().name(format!("x_{sid}_{pid}")));
            x[si].push(v);
        }
    }

    web_sys::console::log_1(&"Creating partner pairs".into());

    // partner pairs (unique)
    let mut seen = HashSet::new();
    let mut pairs = Vec::new();
    for (&sid, student) in students {
        if student.partner.is_some() {
            // partner holds student ID, rename to partner_id
            if let Some(&partner_id) = student.partner.as_ref() {
                if students
                    .get(&partner_id)
                    .and_then(|partner| partner.partner)
                    != Some(sid)
                {
                    continue;
                }

                let (i, j) = if sid.id() < partner_id.id() {
                    (sid, partner_id)
                } else {
                    (partner_id, sid)
                };
                if seen.insert((i, j)) {
                    pairs.push((i, j));
                }
            }
        }
    }

    web_sys::console::log_1(&"Creating weights".into());

    // w_ij_p and same_ij
    let mut w = Vec::new();
    let mut same = Vec::new();
    for &(i, j) in &pairs {
        let mut row = Vec::new();
        for &pid in &project_ids {
            let v = vars.add(variable().binary().name(format!("w_{i}_{j}_{pid}")));
            row.push(v);
        }
        let svar = vars.add(variable().binary().name(format!("same_{i}_{j}")));
        w.push(row);
        same.push(svar);
    }

    web_sys::console::log_1(&"Creating objective".into());

    // Objective
    let mut obj = Expression::from(0.0);

    web_sys::console::log_1(&"Creating wishes".into());
    // wishes
    for (si, &sid) in student_ids.iter().enumerate() {
        let student = &students[&sid];
        if let Some(wishes) = student.wishes {
            for (wi, &project_id) in wishes.iter().enumerate() {
                let pj = project_ids.iter().position(|&x| x == project_id);
                if let Some(pj) = pj {
                    obj += weights[wi] * x[si][pj];
                }
            }
        }
    }
    // partner bonus
    for &svar in &same {
        obj += partner_weight * svar;
    }

    web_sys::console::log_1(&"Building Problem".into());

    // Build problem using MicroLp
    let mut pb = vars.maximise(obj).using(default_solver);

    web_sys::console::log_1(&"Constraint: Student limit".into());

    // each student exactly one
    for (student_projects, (_schueler_id, schueler)) in x.iter().zip(students.iter()) {
        if schueler.ignore {
            log!(format!("Ignore: {}", schueler.name));
            pb = pb.with(student_projects.iter().cloned().sum::<Expression>().eq(0.0));
        } else {
            pb = pb.with(student_projects.iter().cloned().sum::<Expression>().eq(1.0));
        }
    }

    web_sys::console::log_1(&"Constraint: Proj. cap".into());

    // project capacity
    for (pj, &pid) in project_ids.iter().enumerate() {
        let proj = &projects[&pid];
        let sum_p = (0..n).map(|si| x[si][pj]).sum::<Expression>();

        let feste_schueler = students
            .iter()
            .filter(|(_s_id, s)| {
                s.fest
                    .and_then(|fest| s.wishes.map(|w| fest && w[0] == pid))
                    .unwrap_or(false)
            })
            .count() as i32;

        if *proj.teilnehmer.end() != -1 {
            pb = pb.with(
                sum_p
                    .clone()
                    .leq((*proj.teilnehmer.end() + feste_schueler) as f64),
            )
        }
        if *proj.teilnehmer.start() != -1 {
            pb = pb.with(sum_p.geq(Expression::from(*proj.teilnehmer.start() as f64)));
        }
    }

    web_sys::console::log_1(&"Constraint: Stufen".into());

    // Schüler dürfen nur in Projekte, die ihrer Stufe entsprechen, außer sie werden fest zugeordnet
    for (si, &sid) in student_ids.iter().enumerate() {
        let student = &students[&sid];
        for (pj, &pid) in project_ids.iter().enumerate() {
            let project = &projects[&pid];
            // Wenn der Schüler nicht fest zugeordnet ist und die Stufe nicht passt

            let stufe = &student.klasse.stufe();

            if stufe.is_none() {
                continue;
            }
            let stufe = stufe.unwrap();

            if !feste_zuordnung.contains_key(&sid)
                && student.fest != Some(true)
                && !project.stufen.contains(&stufe)
            {
                pb = pb.with(Expression::from(x[si][pj]).eq(0.0));
            }
        }
    }

    web_sys::console::log_1(&"Linearizing".into());

    // partner linearization
    for (k, &(i, j)) in pairs.iter().enumerate() {
        let wrow = &w[k];
        let svar = same[k];
        let sum_w = wrow.iter().cloned().sum::<Expression>();
        pb = pb.with(Expression::from(svar).leq(sum_w));

        for (pj, &_p_id) in project_ids.iter().enumerate() {
            let xi = x[student_ids.iter().position(|&sid| sid == i).unwrap()][pj];
            let xj = x[student_ids.iter().position(|&sid| sid == j).unwrap()][pj];
            let wij = wrow[pj];

            pb = pb
                .with(Expression::from(wij).leq(xi))
                .with(Expression::from(wij).leq(xj))
                .with((xi + xj - 1.0).leq(wij));
        }
    }

    web_sys::console::log_1(&"Feste Zuordnung".into());

    for (s_id, s) in students {
        if Some(true) == s.fest {
            let projekt_id = s.wishes.and_then(|w| w.first().cloned());

            if let Some(projekt_id) = projekt_id {
                let s_idx = student_ids.iter().position(|&sid| sid == *s_id).unwrap();
                let p_idx = project_ids
                    .iter()
                    .position(|&pid| pid == projekt_id)
                    .unwrap();

                log!(format!("{s_id}: {projekt_id}"));

                pb = pb.with(Expression::from(x[s_idx][p_idx]).eq(1.0))
            }
        }
    }

    web_sys::console::log_1(&"Solving".into());

    let solution = pb.solve()?;

    web_sys::console::log_1(&"Solved".into());

    // 1) Build student → project map
    let mut student_assignment: Vec<(SchuelerId, ProjektId)> = Vec::new();
    for (s_idx, (&s_uuid, _student)) in students.iter().enumerate() {
        for (p_idx, &p_id) in project_ids.iter().enumerate() {
            if solution.value(x[s_idx][p_idx]) > 0.5 {
                student_assignment.push((s_uuid, p_id));
                break;
            }
        }
    }

    // 2) Build project → count map
    let mut project_counts: BTreeMap<ProjektId, usize> = BTreeMap::new();
    for (_ref_student, proj_id) in &student_assignment {
        *project_counts.entry(*proj_id).or_default() += 1;
    }

    // 3) Build wish‐rank histogram
    let mut wish_hist: [usize; 5] = [0; 5];
    let mut not_wished_projects: usize = 0;
    for (sid, (&_s_idx, student)) in students.iter().enumerate() {
        if let Some(wishes) = student.wishes {
            let mut in_wishes = false;

            for (wish_rank, &wish_pid) in wishes.iter().enumerate() {
                let p_idx = project_ids.iter().position(|&id| id == wish_pid);
                if let Some(p_idx) = p_idx {
                    // Check if the student is assigned to the project
                    if solution.value(x[sid][p_idx]) > 0.5 {
                        wish_hist[wish_rank] += 1;
                        in_wishes = true;
                        break;
                    }
                }
            }
            if !in_wishes {
                not_wished_projects += 1
            }
        }
    }

    // 4) Number of students who got their partner
    let mut num = 0;
    for (student_uuid, project_id) in student_assignment.iter() {
        let student = students.get(student_uuid);
        if let Some(student) = student {
            if let Some(partner_uuid) = student.partner {
                let partner_project = student_assignment.iter().find_map(|(s_uuid, p_id)| {
                    if s_uuid == &partner_uuid {
                        Some(p_id)
                    } else {
                        None
                    }
                });

                if let Some(partner_project) = partner_project {
                    if partner_project == project_id {
                        num += 1;
                    }
                }
            }
        }
    }

    web_sys::console::log_2(&"Schueler mit Partnern:".to_string().into(), &num.into());

    web_sys::console::log_5(
        &wish_hist[0].into(),
        &wish_hist[1].into(),
        &wish_hist[2].into(),
        &wish_hist[3].into(),
        &wish_hist[4].into(),
    );

    // // ——— Print results ———

    // // 1) Students and their assigned project
    // println!("Student Assignments:");
    // for (student_name, proj_name) in &student_assignment {
    //     println!("- {:<20} → {}", student_name, proj_name);
    // }
    // println!();

    // 2) Projects and how many students each got
    log!("Project Loads:");
    for pid in &project_ids {
        let count = project_counts.get(pid).copied().unwrap_or(0);

        let projekt = projects.get(pid);

        if let Some(projekt) = projekt {
            log!(format!(
                "- {:<20} : {} Schueler von {}-{}",
                projekt.name,
                count,
                projekt.get_min_teilnehmer(),
                projekt.get_max_teilnehmer()
            ));
        }
    }
    println!();

    // 3) Wish‐rank histogram
    log!("Wish Satisfaction:");
    for (i, &count) in wish_hist.iter().enumerate() {
        log!(format!("- {}. wish: {} students", i + 1, count));
    }
    log!(format!("not-wished: {} schueler", not_wished_projects));

    let x = x
        .iter()
        .map(|y| y.iter().map(|v| solution.value(*v)).collect::<Vec<f64>>())
        .collect::<Vec<_>>();

    Ok(x)
}
