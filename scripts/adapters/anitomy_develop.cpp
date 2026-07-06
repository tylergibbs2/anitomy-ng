// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Benchmark adapter for erengy/anitomy (develop branch, C++23, header-only).
// Reads one filename per line on stdin; writes one JSON object per line:
//   {"input": "<line>", "output": {"<kind>": ["<value>", ...], ...}}
// Kinds are emitted as current-schema snake_case names (schema=current), which
// already match this branch's ElementKind. See scripts/benchmark.py.
//
// Build (see .github/workflows/benchmark.yml):
//   g++-14 -std=c++23 -O2 -I<anitomy>/include anitomy_develop.cpp -o adapter

#include <cstdio>
#include <iostream>
#include <map>
#include <string>
#include <vector>

#include <anitomy.hpp>

static const char* kind_name(anitomy::ElementKind kind) {
  using enum anitomy::ElementKind;
  switch (kind) {
    case AudioTerm: return "audio_term";
    case Device: return "device";
    case Episode: return "episode";
    case EpisodeTitle: return "episode_title";
    case FileChecksum: return "file_checksum";
    case FileExtension: return "file_extension";
    case Language: return "language";
    case Other: return "other";
    case Part: return "part";
    case ReleaseGroup: return "release_group";
    case ReleaseInformation: return "release_information";
    case ReleaseVersion: return "release_version";
    case Season: return "season";
    case Source: return "source";
    case Subtitles: return "subtitles";
    case Title: return "title";
    case Type: return "type";
    case VideoResolution: return "video_resolution";
    case VideoTerm: return "video_term";
    case Volume: return "volume";
    case Year: return "year";
  }
  return "?";
}

static void json_escape(const std::string& s, std::string& out) {
  for (unsigned char c : s) {
    switch (c) {
      case '"': out += "\\\""; break;
      case '\\': out += "\\\\"; break;
      case '\n': out += "\\n"; break;
      case '\r': out += "\\r"; break;
      case '\t': out += "\\t"; break;
      default:
        if (c < 0x20) {
          char buf[8];
          std::snprintf(buf, sizeof buf, "\\u%04x", c);
          out += buf;
        } else {
          out += static_cast<char>(c);  // pass UTF-8 bytes through unchanged
        }
    }
  }
}

int main() {
  std::string line;
  while (std::getline(std::cin, line)) {
    std::map<std::string, std::vector<std::string>> grouped;
    for (const auto& el : anitomy::parse(line)) {
      grouped[kind_name(el.kind)].push_back(el.value);
    }

    std::string out = "{\"input\":\"";
    json_escape(line, out);
    out += "\",\"output\":{";
    bool first = true;
    for (const auto& [kind, values] : grouped) {
      if (!first) out += ",";
      first = false;
      out += "\"";
      json_escape(kind, out);
      out += "\":[";
      for (size_t i = 0; i < values.size(); ++i) {
        if (i) out += ",";
        out += "\"";
        json_escape(values[i], out);
        out += "\"";
      }
      out += "]";
    }
    out += "}}";
    std::cout << out << "\n";
  }
  return 0;
}
