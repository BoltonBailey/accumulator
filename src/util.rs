use crate::group::Group;
use rug::Integer;

/// Poor man's type-level programming.
/// This trait allows us to reflect "type-level" (i.e. static) information at runtime.
pub trait TypeRep: 'static {
  type Rep: 'static;
  fn rep() -> &'static Self::Rep;
}

#[derive(Debug)]
pub enum UtilityError {
  NoSolutionToLinearCongruence,
}

pub fn int<T>(val: T) -> Integer
where
  Integer: From<T>,
{
  Integer::from(val)
}

/// Computes the `(xy)`th root of `g` given the `x`th and `y`th roots of `g` and `(x, y)` coprime.
/// Consider moving this to accumulator?
#[allow(clippy::similar_names)]
pub fn shamir_trick<G: Group>(
  xth_root: &G::Elem,
  yth_root: &G::Elem,
  x: &Integer,
  y: &Integer,
) -> Option<G::Elem> {
  if G::exp(xth_root, x) != G::exp(yth_root, y) {
    return None;
  }

  let (gcd, a, b) = <(Integer, Integer, Integer)>::from(x.gcd_cofactors_ref(&y));

  if gcd != int(1) {
    return None;
  }

  Some(G::op(&G::exp(xth_root, &b), &G::exp(yth_root, &a)))
}

// Solve a linear congruence of form `ax = b mod m` for the set of solutions x,
// characterized by integers mu and v such that x = mu + vn where n is any integer.
pub fn solve_linear_congruence(
  a: &Integer,
  b: &Integer,
  m: &Integer,
) -> Result<(Integer, Integer), UtilityError> {
  // g = gcd(a, m) => da + em = g
  let (g, d, _) = a.clone().gcd_cofactors(m.clone(), Integer::new());

  // q = floor_div(b, g)
  // r = b % g
  let (q, r) = b.clone().div_rem_floor(g.clone());
  if r != Integer::from(0) {
    return Err(UtilityError::NoSolutionToLinearCongruence);
  }

  // mu = (q * d) % m
  // v = m / g
  let mu = (q * d) % m;
  let (v, _) = m.clone().div_rem_floor(g);
  Ok((mu, v))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::group::{Group, Rsa2048, UnknownOrderGroup};
  use crate::util::int;

  #[test]
  fn test_linear_congruence_solver() {
    assert_eq!(
      (Integer::from(-2), Integer::from(4)),
      solve_linear_congruence(&Integer::from(3), &Integer::from(2), &Integer::from(4)).unwrap()
    );

    assert_eq!(
      (Integer::from(-2), Integer::from(4)),
      solve_linear_congruence(&Integer::from(3), &Integer::from(2), &Integer::from(4)).unwrap()
    );

    assert_eq!(
      (Integer::from(1), Integer::from(2)),
      solve_linear_congruence(&Integer::from(5), &Integer::from(1), &Integer::from(2)).unwrap()
    );

    assert_eq!(
      (Integer::from(-3), Integer::from(5)),
      solve_linear_congruence(&Integer::from(2), &Integer::from(4), &Integer::from(5)).unwrap()
    );

    assert_eq!(
      (Integer::from(2491), Integer::from(529)),
      solve_linear_congruence(
        &Integer::from(230),
        &Integer::from(1081),
        &Integer::from(12167)
      )
      .unwrap()
    );
  }

  #[test]
  fn test_linear_congruence_solver_no_solution() {
    // Let g = gcd(a, m). If b is not divisible by g, there are no solutions. If b is divisible by
    // g, there are g solutions.
    let result =
      solve_linear_congruence(&Integer::from(33), &Integer::from(7), &Integer::from(143));
    assert!(result.is_err());

    let result =
      solve_linear_congruence(&Integer::from(13), &Integer::from(14), &Integer::from(39));
    assert!(result.is_err());
  }

  #[test]
  fn test_shamir_trick() {
    let (x, y, z) = (&int(13), &int(17), &int(19));
    let xth_root = Rsa2048::exp(&Rsa2048::unknown_order_elem(), &int(y * z));
    let yth_root = Rsa2048::exp(&Rsa2048::unknown_order_elem(), &int(x * z));
    let xyth_root = Rsa2048::exp(&Rsa2048::unknown_order_elem(), z);
    assert!(shamir_trick::<Rsa2048>(&xth_root, &yth_root, x, y) == Some(xyth_root));
  }

  #[test]
  fn test_shamir_trick_failure() {
    let (x, y, z) = (&int(7), &int(14), &int(19)); // Inputs not coprime.
    let xth_root = Rsa2048::exp(&Rsa2048::unknown_order_elem(), &int(y * z));
    let yth_root = Rsa2048::exp(&Rsa2048::unknown_order_elem(), &int(x * z));
    assert!(shamir_trick::<Rsa2048>(&xth_root, &yth_root, x, y) == None);
  }
}
