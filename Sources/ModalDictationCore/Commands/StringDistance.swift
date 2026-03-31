import Foundation

enum StringDistance {

    static func soundex(_ string: String) -> String {
        let chars = Array(string.lowercased().unicodeScalars)
        let aToZ = Unicode.Scalar("a").value...Unicode.Scalar("z").value
        guard let first = chars.first, aToZ.contains(first.value) else {
            return ""
        }

        var code = String(Character(first)).uppercased()
        var lastDigit = soundexDigit(first)

        for scalar in chars.dropFirst() {
            guard aToZ.contains(scalar.value) else { continue }
            let digit = soundexDigit(scalar)
            guard let d = digit, d != lastDigit else {
                if digit == nil { lastDigit = nil }
                continue
            }
            code.append(String(d))
            lastDigit = digit
            if code.count == 4 { break }
        }

        while code.count < 4 { code.append("0") }
        return code
    }

    static func levenshtein(_ a: String, _ b: String) -> Int {
        let a = Array(a.lowercased())
        let b = Array(b.lowercased())
        if a.isEmpty { return b.count }
        if b.isEmpty { return a.count }

        var prev = Array(0...b.count)
        var curr = [Int](repeating: 0, count: b.count + 1)

        for i in 1...a.count {
            curr[0] = i
            for j in 1...b.count {
                let cost = a[i - 1] == b[j - 1] ? 0 : 1
                curr[j] = min(prev[j] + 1, curr[j - 1] + 1, prev[j - 1] + cost)
            }
            swap(&prev, &curr)
        }
        return prev[b.count]
    }

    static let maxEditDistance = 2
}

private extension StringDistance {
    static func soundexDigit(_ c: Unicode.Scalar) -> Character? {
        switch Character(c) {
        case "b", "f", "p", "v": return "1"
        case "c", "g", "j", "k", "q", "s", "x", "z": return "2"
        case "d", "t": return "3"
        case "l": return "4"
        case "m", "n": return "5"
        case "r": return "6"
        default: return nil
        }
    }
}
