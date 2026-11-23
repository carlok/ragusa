//! Jabłonowski's catalogue of inequalities between integer knot invariants (arXiv:2605.22652),
//! as transcribed in `engine/data/knots/jablonowski2026.json`.
//!
//! The catalogue is a graph: vertex `u` above vertex `v` means `u ≥ v` for every non-trivial knot,
//! where each vertex is a quantity such as "genus, doubled". Chains of relations are relations, so
//! the oracle uses the transitive closure. Only some vertices correspond to KnotInfo columns; a
//! mapped vertex is `scale·column + offset`, or `scale·|column| + offset`. Every implied relation
//! between two mapped vertices is stated as linear bounds on columns, one per column as target:
//!
//! - `s_u·X + o_u ≥ s_v·Y + o_v` gives `Y ≤ (s_u/s_v)·X + (o_u − o_v)/s_v` and
//!   `X ≥ (s_v/s_u)·Y + (o_v − o_u)/s_u`;
//! - when the smaller side is `|Y|`, both signs of `Y` hold, giving two more bounds;
//! - when the larger side is an absolute value the relation is not linear and is skipped (counted).
//!
//! The catalogue's linear sharper forms are added as given and solved for their other column.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use ragusa_gen::{Bound, Op, Rational, ALWAYS};
use serde_json::Value;

pub const STANDARD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/knots/jablonowski2026.json"
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantity {
    pub column: String,
    pub scale: i128,
    pub offset: i128,
    pub absolute: bool,
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub id: String,
    pub name: String,
    pub quantity: Option<Quantity>,
    /// `exact`, `likely`, or why there is no column.
    pub mapping: String,
}

#[derive(Clone, Debug)]
pub struct Relation {
    pub id: String,
    /// `source ≥ target`, as vertex indices.
    pub source: usize,
    pub target: usize,
    pub reference: String,
}

#[derive(Clone, Debug)]
pub struct CatalogueBound {
    /// Hypothesis [`ALWAYS`]: the bound is claimed for every non-trivial knot.
    pub bound: Bound,
    pub derivation: String,
    /// Some column involved is mapped to its vertex only as "likely".
    pub likely: bool,
}

pub struct Catalogue {
    pub citation: String,
    pub vertices: Vec<Vertex>,
    pub relations: Vec<Relation>,
    /// Sharper forms given as linear bounds on columns, with their ids.
    pub sharper: Vec<(String, Bound)>,
}

fn string(x: &Value, what: &str) -> Result<String, String> {
    x.as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{what}: expected a string, got {x}"))
}

fn integer(x: &Value, what: &str) -> Result<i128, String> {
    x.as_i64()
        .map(i128::from)
        .ok_or_else(|| format!("{what}: expected an integer, got {x}"))
}

fn ratio(r: Rational) -> (i128, i128) {
    r.as_ratio()
        .expect("catalogue coefficients are small fractions")
}

fn neg(r: Rational) -> Rational {
    let (p, q) = ratio(r);
    Rational::new(-p, q)
}

/// `T op a·F + b` solved for `F`; `None` unless the bound has exactly one term.
pub fn reverse(bound: &Bound) -> Option<Bound> {
    let [(feature, a)] = bound.terms.as_slice() else {
        return None;
    };
    let ((p, q), (bp, bq)) = (ratio(*a), ratio(bound.constant));
    let op = match (bound.op, p > 0) {
        (Op::Le, true) | (Op::Ge, false) => Op::Ge,
        _ => Op::Le,
    };
    Some(Bound::new(
        bound.hypothesis.clone(),
        feature.clone(),
        op,
        [(bound.target.clone(), Rational::new(q, p))],
        Rational::new(-(bp * q), bq * p),
    ))
}

impl Catalogue {
    pub fn standard() -> Result<Catalogue, String> {
        Catalogue::load(Path::new(STANDARD))
    }

    pub fn load(path: &Path) -> Result<Catalogue, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Catalogue::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Catalogue, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let mut vertices = Vec::new();
        for x in v["vertices"].as_array().ok_or("no vertices")? {
            let id = string(&x["id"], "vertex id")?;
            let quantity = match &x["quantity"] {
                Value::Null => None,
                q => {
                    let quantity = Quantity {
                        column: string(&q["column"], &id)?,
                        scale: integer(&q["scale"], &id)?,
                        offset: integer(&q["offset"], &id)?,
                        absolute: q["absolute"].as_bool().unwrap_or(false),
                    };
                    if quantity.scale <= 0 {
                        return Err(format!("{id}: scale must be positive"));
                    }
                    Some(quantity)
                }
            };
            vertices.push(Vertex {
                name: string(&x["name"], &id)?,
                mapping: string(&x["mapping"], &id)?,
                id,
                quantity,
            });
        }

        let index: HashMap<&str, usize> = vertices
            .iter()
            .enumerate()
            .map(|(k, x)| (x.id.as_str(), k))
            .collect();
        let vertex = |x: &Value| -> Result<usize, String> {
            let id = string(x, "relation endpoint")?;
            index
                .get(id.as_str())
                .copied()
                .ok_or_else(|| format!("unknown vertex {id:?}"))
        };
        let relations = v["relations"]
            .as_array()
            .ok_or("no relations")?
            .iter()
            .map(|x| {
                Ok(Relation {
                    id: x["id"].to_string(),
                    source: vertex(&x["source"])?,
                    target: vertex(&x["target"])?,
                    reference: string(&x["ref"], "ref")?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut sharper = Vec::new();
        for x in v["sharper_forms_not_in_graph"]
            .as_array()
            .ok_or("no sharper forms")?
        {
            let Some(l) = x.get("linear") else {
                continue;
            };
            let id = string(&x["id"], "sharper form id")?;
            let op = match l["op"].as_str() {
                Some("<=") => Op::Le,
                Some(">=") => Op::Ge,
                _ => return Err(format!("{id}: bad op")),
            };
            let terms = l["terms"]
                .as_object()
                .ok_or_else(|| format!("{id}: no terms"))?
                .iter()
                .map(|(f, c)| Ok((f.clone(), string(c, &id)?.parse::<Rational>()?)))
                .collect::<Result<Vec<_>, String>>()?;
            let constant = string(&l["constant"], &id)?.parse::<Rational>()?;
            let target = string(&l["target"], &id)?;
            sharper.push((id, Bound::new(ALWAYS, target, op, terms, constant)));
        }

        Ok(Catalogue {
            citation: string(&v["source"]["citation"], "citation")?,
            vertices,
            relations,
            sharper,
        })
    }

    /// Columns some vertex is mapped to.
    pub fn columns(&self) -> HashSet<&str> {
        self.vertices
            .iter()
            .filter_map(|x| x.quantity.as_ref().map(|q| q.column.as_str()))
            .collect()
    }

    /// Columns mapped under an absolute value: their sign is not fixed.
    pub fn signed_columns(&self) -> HashSet<&str> {
        self.vertices
            .iter()
            .filter_map(|x| x.quantity.as_ref())
            .filter(|q| q.absolute)
            .map(|q| q.column.as_str())
            .collect()
    }

    fn column_is_likely(&self, column: &str) -> bool {
        self.vertices.iter().any(|x| {
            x.mapping != "exact" && x.quantity.as_ref().is_some_and(|q| q.column == column)
        })
    }

    /// Every ordered pair of distinct vertices `(u, v)` with `u ≥ v` implied by a chain of
    /// relations, with the relation indices along one shortest chain.
    pub fn closure(&self) -> Vec<(usize, usize, Vec<usize>)> {
        let n = self.vertices.len();
        let mut out = Vec::new();
        for u in 0..n {
            let mut via: Vec<Option<usize>> = vec![None; n];
            let mut seen = vec![false; n];
            seen[u] = true;
            let mut queue = VecDeque::from([u]);
            while let Some(x) = queue.pop_front() {
                for (e, rel) in self.relations.iter().enumerate() {
                    if rel.source == x && !seen[rel.target] {
                        seen[rel.target] = true;
                        via[rel.target] = Some(e);
                        queue.push_back(rel.target);
                    }
                }
            }
            for v in (0..n).filter(|&v| v != u && seen[v]) {
                let mut path = Vec::new();
                let mut x = v;
                while x != u {
                    let e = via[x].expect("reached vertices have a predecessor");
                    path.push(e);
                    x = self.relations[e].source;
                }
                path.reverse();
                out.push((u, v, path));
            }
        }
        out
    }

    /// The catalogue as distinct bounds on columns (first derivation kept), and the number of
    /// implied relations between mapped vertices skipped because their larger side is an absolute
    /// value.
    pub fn bounds(&self) -> (Vec<CatalogueBound>, usize) {
        let mut out: Vec<CatalogueBound> = Vec::new();
        let mut skipped = 0;
        for (u, v, path) in self.closure() {
            let (Some(qu), Some(qv)) = (&self.vertices[u].quantity, &self.vertices[v].quantity)
            else {
                continue;
            };
            if qu.absolute {
                skipped += 1;
                continue;
            }
            let ids: Vec<&str> = path
                .iter()
                .map(|&e| self.relations[e].id.as_str())
                .collect();
            let derivation = format!(
                "{} >= {} (relation{} {})",
                self.vertices[u].id,
                self.vertices[v].id,
                if ids.len() == 1 { "" } else { "s" },
                ids.join(", ")
            );
            let likely = self.column_is_likely(&qu.column) || self.column_is_likely(&qv.column);
            let mut push = |target: &str, op: Op, feature: &str, c: Rational, k: Rational| {
                out.push(CatalogueBound {
                    bound: Bound::new(ALWAYS, target, op, [(feature.to_string(), c)], k),
                    derivation: derivation.clone(),
                    likely,
                });
            };
            let (a, b) = (
                Rational::new(qu.scale, qv.scale),
                Rational::new(qu.offset - qv.offset, qv.scale),
            );
            let (a2, b2) = (
                Rational::new(qv.scale, qu.scale),
                Rational::new(qv.offset - qu.offset, qu.scale),
            );
            push(&qv.column, Op::Le, &qu.column, a, b);
            push(&qu.column, Op::Ge, &qv.column, a2, b2);
            if qv.absolute {
                push(&qv.column, Op::Ge, &qu.column, neg(a), neg(b));
                push(&qu.column, Op::Ge, &qv.column, neg(a2), b2);
            }
        }
        for (id, bound) in &self.sharper {
            let likely = std::iter::once(bound.target.as_str())
                .chain(bound.terms.iter().map(|(f, _)| f.as_str()))
                .any(|c| self.column_is_likely(c));
            let derivation = format!("sharper form {id}");
            for b in std::iter::once(bound.clone()).chain(reverse(bound)) {
                out.push(CatalogueBound {
                    bound: b,
                    derivation: derivation.clone(),
                    likely,
                });
            }
        }
        let mut seen = HashSet::new();
        out.retain(|c| seen.insert(c.bound.clone()));
        (out, skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find<'a>(bounds: &'a [CatalogueBound], text: &str) -> Option<&'a CatalogueBound> {
        bounds.iter().find(|b| b.bound.to_string() == text)
    }

    #[test]
    fn standard_catalogue_parses() {
        let c = Catalogue::standard().unwrap();
        assert_eq!(c.vertices.len(), 33);
        assert_eq!(c.relations.len(), 47);
        assert_eq!(c.columns().len(), 13);
        assert_eq!(
            c.signed_columns(),
            HashSet::from([
                "signature",
                "rasmussen_invariant",
                "ozsvath_szabo_tau_invariant"
            ])
        );
        assert_eq!(c.sharper.len(), 1);
    }

    #[test]
    fn closure_follows_chains() {
        let c = Catalogue::standard().unwrap();
        let id = |s: &str| c.vertices.iter().position(|x| x.id == s).unwrap();
        let closure = c.closure();
        let (_, _, path) = closure
            .iter()
            .find(|(u, v, _)| *u == id("c") && *v == id("2u"))
            .unwrap();
        let ids: Vec<&str> = path.iter().map(|&e| c.relations[e].id.as_str()).collect();
        assert_eq!(ids, ["24", "32"]);
        assert!(closure.iter().all(|(u, v, _)| u != v));
    }

    #[test]
    fn bounds_on_columns() {
        let c = Catalogue::standard().unwrap();
        let (bounds, skipped) = c.bounds();
        // No absolute-valued vertex has an outgoing relation, so nothing is skipped.
        assert_eq!(skipped, 0);
        for text in [
            "(TRUE) → (unknotting_number <= 1/2·crossing_number)",
            "(TRUE) → (crossing_number >= 2·unknotting_number)",
            "(TRUE) → (signature <= 2·smooth_four_genus)",
            "(TRUE) → (signature >= -2·smooth_four_genus)",
            "(TRUE) → (smooth_four_genus >= 1/2·signature)",
            "(TRUE) → (smooth_four_genus >= -1/2·signature)",
            "(TRUE) → (unknotting_number <= 1/2·crossing_number + -1/2)",
            "(TRUE) → (crossing_number >= 2·unknotting_number + 1)",
            "(TRUE) → (braid_index <= 1/2·arc_index)",
        ] {
            assert!(find(&bounds, text).is_some(), "missing {text}");
        }
        assert!(find(&bounds, "(TRUE) → (braid_index <= 1/2·arc_index)")
            .unwrap()
            .derivation
            .contains("relation 14"));
        assert!(
            find(&bounds, "(TRUE) → (three_genus <= 1·td_clasp_number)")
                .unwrap()
                .likely
        );
    }
}
