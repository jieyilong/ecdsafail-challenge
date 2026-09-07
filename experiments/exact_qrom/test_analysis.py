import math
import unittest

from analyze import exact_upper


class ExactBoundTests(unittest.TestCase):
    def test_zero_failures(self):
        expected = -math.expm1(math.log(0.05) / 99998)
        self.assertAlmostEqual(exact_upper(0, 99998), expected, places=12)

    def test_two_failures_binomial_equation(self):
        n = 99998
        p = exact_upper(2, n)
        odds = p / (1 - p)
        cdf = math.exp(n * math.log1p(-p)) * (1 + n * odds + n * (n - 1) * odds * odds / 2)
        self.assertAlmostEqual(cdf, 0.05, places=12)

    def test_monotonicity(self):
        self.assertLess(exact_upper(0, 99998), exact_upper(1, 99998))
        self.assertLess(exact_upper(1, 99998), exact_upper(2, 99998))


if __name__ == "__main__":
    unittest.main()
