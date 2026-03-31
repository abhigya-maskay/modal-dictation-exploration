import Foundation

enum NumberParser {

    static func parse(from tokens: [String], startingAt start: Int) -> (value: Int, tokensConsumed: Int)? {
        guard start < tokens.count else { return nil }

        if let digit = Int(tokens[start]) {
            return (digit, 1)
        }

        let word = tokens[start].lowercased()

        if start + 1 < tokens.count && tokens[start + 1].lowercased() == "hundred" {
            if let ones = smallNumbers[word], ones <= 9 {
                return (ones * 100, 2)
            }
        }

        if let value = smallNumbers[word] { return (value, 1) }
        if let value = decadeNumbers[word] { return (value, 1) }

        return nil
    }

    private static let smallNumbers: [String: Int] = [
        "one": 1, "two": 2, "three": 3, "four": 4, "five": 5,
        "six": 6, "seven": 7, "eight": 8, "nine": 9, "ten": 10,
    ]

    private static let decadeNumbers: [String: Int] = [
        "twenty": 20, "thirty": 30, "forty": 40, "fifty": 50,
        "sixty": 60, "seventy": 70, "eighty": 80, "ninety": 90,
    ]
}
