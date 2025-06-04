// pub mod custom_constraints;

// use csv::ReaderBuilder;
use crate::{
    Projekt,
    types::{ProjektId, SaveFileSchueler, SchuelerId},
};
use good_lp::{
    Expression, ProblemVariables, ResolutionError, Solution, default_solver,
    solvers::{SolverModel, microlp::MicroLpSolution},
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
) -> Result<(MicroLpSolution, Vec<Vec<f64>>), ResolutionError> {
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
    for student_projects in &x {
        pb = pb.with(student_projects.iter().cloned().sum::<Expression>().eq(1.0));
    }

    web_sys::console::log_1(&"Constraint: Proj. cap".into());

    // project capacity
    for (pj, &pid) in project_ids.iter().enumerate() {
        let proj = &projects[&pid];
        let sum_p = (0..n).map(|si| x[si][pj]).sum::<Expression>();
        if *proj.teilnehmer.end() != -1 {
            pb = pb.with(sum_p.clone().leq(*proj.teilnehmer.end() as f64))
        }
        if *proj.teilnehmer.start() != -1 {
            pb = pb.with(sum_p.geq(Expression::from(*proj.teilnehmer.start() as f64)));
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

    // feste zuordnung: Student 100% in that project
    for (s, p) in feste_zuordnung.iter() {
        let s_idx = student_ids.iter().position(|&sid| sid == *s).unwrap();
        let p_idx = project_ids.iter().position(|&pid| pid == *p).unwrap();
        pb = pb.with(Expression::from(x[s_idx][p_idx]).eq(1.0));
    }

    web_sys::console::log_1(&"Solving".into());

    let solution = pb.solve()?;

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
    for (sid, (&_s_idx, student)) in students.iter().enumerate() {
        if let Some(wishes) = student.wishes {
            for (wish_rank, &wish_pid) in wishes.iter().enumerate() {
                let p_idx = project_ids.iter().position(|&id| id == wish_pid);
                if let Some(p_idx) = p_idx {
                    // Check if the student is assigned to the project
                    if solution.value(x[sid][p_idx]) > 0.5 {
                        wish_hist[wish_rank] += 1;
                        break;
                    }
                }
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

    // // 2) Projects and how many students each got
    // println!("Project Loads:");
    // for pid in &project_ids {
    //     let name = &projects[pid].name;
    //     let count = project_counts.get(name).copied().unwrap_or(0);
    //     println!("- {:<20} : {} students", name, count);
    // }
    // println!();

    // // 3) Wish‐rank histogram
    // println!("Wish Satisfaction:");
    // for (i, &count) in wish_hist.iter().enumerate() {
    //     println!("- {}. wish: {} students", i + 1, count);
    // }

    let x = x
        .iter()
        .map(|y| y.iter().map(|v| solution.value(*v)).collect::<Vec<f64>>())
        .collect::<Vec<_>>();

    Ok((solution, x))
}
