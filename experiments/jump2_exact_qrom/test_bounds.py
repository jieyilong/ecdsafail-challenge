import math
import unittest

from analyze import binomial_cdf, invert_cdf, conservative_difference_upper


class BoundsTests(unittest.TestCase):
    def test_zero_failure_upper(self):
        expected=-math.expm1(math.log(0.025)/100000)
        self.assertAlmostEqual(invert_cdf(0,100000,0.025),expected,places=12)

    def test_known_small_cdf(self):
        self.assertAlmostEqual(binomial_cdf(2,5,0.5),0.5,places=12)

    def test_rare_tail_equation(self):
        p=invert_cdf(30,100000,0.025)
        self.assertAlmostEqual(binomial_cdf(30,100000,p),0.025,places=10)

    def test_conservative_difference(self):
        bound=conservative_difference_upper(2,3,99998)
        self.assertGreater(bound,(2-3)/99998)
        self.assertLess(bound,0.0005)


if __name__=="__main__": unittest.main()
