import Testing
@testable import ModalDictationCore

@Suite("PhraseTrie")
struct PhraseTrieTests {

    @Test func test_longestMatch_prefersLongerPhrase() {
        let trie = PhraseTrie(commands: .fixture(keys: ["f": "f", "f twelve": "f12"]))

        let result = trie.longestMatch(in: ["f", "twelve"], startingAt: 0)

        #expect(result?.length == 2)
        #expect(result?.category == .key("f12"))
    }
}

@Suite("CommandMatcher")
struct CommandMatcherTests {

    @Test func test_match_singleKey() {
        let matcher = CommandMatcher(commands: .fixture(keys: ["escape": "escape"]))

        let result = matcher.match("escape")

        #expect(result == [.keystroke(key: "escape", modifiers: [], repeat: 1)])
    }

    @Test func test_match_modifierAccumulation() {
        let matcher = CommandMatcher(commands: .fixture(
            modifiers: ["command": "cmd", "shift": "shift"],
            keys: ["adam": "a"]
        ))

        let result = matcher.match("command shift adam")

        #expect(result == [.keystroke(key: "a", modifiers: ["cmd", "shift"], repeat: 1)])
    }

    @Test func test_match_action() {
        let matcher = CommandMatcher(commands: .fixture(actions: ["dictation mode": "mode:dictation"]))

        let result = matcher.match("dictation mode")

        #expect(result == [.action("mode:dictation")])
    }

    @Test func test_match_unrecognizedTokens_skipped() {
        let matcher = CommandMatcher(commands: .fixture(keys: ["up": "up"]))

        let result = matcher.match("hello there up world")

        #expect(result == [.keystroke(key: "up", modifiers: [], repeat: 1)])
    }

    @Test func test_match_danglingModifiers_discarded() {
        let matcher = CommandMatcher(commands: .fixture(modifiers: ["command": "cmd"]))

        let result = matcher.match("command")

        #expect(result == [])
    }

    @Test func test_match_modifiersBeforeAction_discarded() {
        let matcher = CommandMatcher(commands: .fixture(
            actions: ["select all": "selectAll"],
            modifiers: ["command": "cmd"]
        ))

        let result = matcher.match("command select all")

        #expect(result == [.action("selectAll")])
    }

    @Test func test_match_repeatWithNumberWord() {
        let matcher = CommandMatcher(commands: .fixture(keys: ["down": "down"]))

        let result = matcher.match("down five times")

        #expect(result == [.keystroke(key: "down", modifiers: [], repeat: 5)])
    }

    @Test func test_match_repeatWithDigitString() {
        let matcher = CommandMatcher(commands: .fixture(keys: ["down": "down"]))

        let result = matcher.match("down 3 times")

        #expect(result == [.keystroke(key: "down", modifiers: [], repeat: 3)])
    }

    @Test func test_match_repeatWithHundred() {
        let matcher = CommandMatcher(commands: .fixture(keys: ["down": "down"]))

        let result = matcher.match("down two hundred times")

        #expect(result == [.keystroke(key: "down", modifiers: [], repeat: 200)])
    }

    @Test func test_match_fuzzyFallback_matchesCloseWord() {
        let matcher = CommandMatcher(commands: .fixture(keys: ["adam": "a"]))

        let result = matcher.match("atum")

        #expect(result == [.keystroke(key: "a", modifiers: [], repeat: 1)])
    }

    @Test func test_match_fuzzyFallback_rejectsShortWordFalsePositive() {
        let matcher = CommandMatcher(commands: .fixture(keys: ["up": "up"]))

        let result = matcher.match("go")

        #expect(result == [])
    }
}

@Suite("StringDistance")
struct StringDistanceTests {

    @Test func test_soundex_phoneticEquivalence() {
        #expect(StringDistance.soundex("adam") == StringDistance.soundex("atum"))
        #expect(StringDistance.soundex("adam") != StringDistance.soundex("zebra"))
    }

    @Test func test_levenshtein_distances() {
        #expect(StringDistance.levenshtein("kitten", "sitting") == 3)
        #expect(StringDistance.levenshtein("same", "same") == 0)
        #expect(StringDistance.levenshtein("", "abc") == 3)
    }
}
