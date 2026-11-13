"""Synthetic fixtures only: no generated game data is included."""

import unittest

from inspect_types import HEADER, inspect


CATALOGUE = HEADER + """
//! reflection data of a synthetic fixture.
    /// `keen::Example`: 8 bytes, aligned to 4. Left in padding, as Rust cannot place them there: `hidden`.
    pub struct Example {
        /// `count`, of `keen::uint32`, at 0x4.
        pub count: u32,
    }
    /// `keen::ecs::Example`: 1 bytes, aligned to 1.
    pub struct Example { }
    /// `keen::Choice`: an enum stored as `u8`.
    pub struct Choice(pub u8);
    /// `keen::Alias`, another name for `keen::uint32`.
    pub type Alias = u32;
"""


class CatalogueTests(unittest.TestCase):
    def test_distinguishes_declarations_and_keeps_qualified_names(self):
        result = inspect(CATALOGUE)
        self.assertEqual(result["counts"], {"struct": 2, "enum": 1, "alias": 1})
        self.assertEqual(result["total_types"], 4)
        self.assertEqual(result["structs_with_padding_warnings"], 1)
        self.assertEqual(result["types"][1]["name"], "keen::ecs::Example")
        self.assertEqual(result["types"][1]["fields"], [])
        self.assertEqual(result["types"][0]["fields"], [
            {"name": "count", "type": "keen::uint32", "offset": "0x4"}
        ])

    def test_bounded_case_insensitive_search_reports_total(self):
        result = inspect(CATALOGUE, ["EXAMPLE"], limit=1)
        self.assertEqual(result["matched"], 2)
        self.assertTrue(result["truncated"])
        self.assertEqual(len(result["types"]), 1)
        self.assertEqual(inspect(CATALOGUE, ["absent"])["matched"], 0)

    def test_refuses_wrong_source_and_changed_format(self):
        for text in ["arbitrary Rust", HEADER, HEADER + '\n/// `keen::Future`: unknown format.']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                inspect(text)


if __name__ == "__main__":
    unittest.main()
