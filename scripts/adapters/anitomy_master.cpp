// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Benchmark adapter for erengy/anitomy (master branch, C++14, wchar_t-based).
// Reads one filename per line on stdin; writes one JSON object per line:
//   {"input": "<line>", "output": {"<category>": ["<value>", ...], ...}}
// Categories are emitted with this branch's old snake_case names
// (schema=old); scripts/benchmark.py maps them to current ElementKind via
// build_fixtures.OLD_KEY_MAP.
//
// Build (see .github/workflows/benchmark.yml): this branch is NOT header-only,
// so its sources are compiled in:
//   g++ -std=c++14 -O2 -I<anitomy> anitomy_master.cpp <anitomy>/anitomy/*.cpp -o adapter

#include <codecvt>
#include <cstdio>
#include <iostream>
#include <locale>
#include <map>
#include <string>
#include <vector>

#include <anitomy/anitomy.h>

static std::string to_utf8(const std::wstring& w) {
  std::wstring_convert<std::codecvt_utf8<wchar_t>> conv;
  return conv.to_bytes(w);
}

static std::wstring from_utf8(const std::string& s) {
  std::wstring_convert<std::codecvt_utf8<wchar_t>> conv;
  return conv.from_bytes(s);
}

static const char* category_name(anitomy::ElementCategory category) {
  using namespace anitomy;
  switch (category) {
    case kElementAnimeSeason: return "anime_season";
    case kElementAnimeSeasonPrefix: return "anime_season_prefix";
    case kElementAnimeTitle: return "anime_title";
    case kElementAnimeType: return "anime_type";
    case kElementAnimeYear: return "anime_year";
    case kElementAudioTerm: return "audio_term";
    case kElementDeviceCompatibility: return "device_compatibility";
    case kElementEpisodeNumber: return "episode_number";
    case kElementEpisodeNumberAlt: return "episode_number_alt";
    case kElementEpisodePrefix: return "episode_prefix";
    case kElementEpisodeTitle: return "episode_title";
    case kElementFileChecksum: return "file_checksum";
    case kElementFileExtension: return "file_extension";
    case kElementFileName: return "file_name";
    case kElementLanguage: return "language";
    case kElementOther: return "other";
    case kElementReleaseGroup: return "release_group";
    case kElementReleaseInformation: return "release_information";
    case kElementReleaseVersion: return "release_version";
    case kElementSource: return "source";
    case kElementSubtitles: return "subtitles";
    case kElementVideoResolution: return "video_resolution";
    case kElementVideoTerm: return "video_term";
    case kElementVolumeNumber: return "volume_number";
    case kElementVolumePrefix: return "volume_prefix";
    default: return nullptr;  // iterate sentinels / kElementUnknown
  }
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
          out += static_cast<char>(c);
        }
    }
  }
}

int main() {
  std::string line;
  while (std::getline(std::cin, line)) {
    anitomy::Anitomy parser;
    parser.Parse(from_utf8(line));

    std::map<std::string, std::vector<std::string>> grouped;
    for (const auto& element : parser.elements()) {
      const char* name = category_name(element.first);
      if (name) {
        grouped[name].push_back(to_utf8(element.second));
      }
    }

    std::string out = "{\"input\":\"";
    json_escape(line, out);
    out += "\",\"output\":{";
    bool first = true;
    for (const auto& kv : grouped) {
      if (!first) out += ",";
      first = false;
      out += "\"";
      json_escape(kv.first, out);
      out += "\":[";
      for (size_t i = 0; i < kv.second.size(); ++i) {
        if (i) out += ",";
        out += "\"";
        json_escape(kv.second[i], out);
        out += "\"";
      }
      out += "]";
    }
    out += "}}";
    std::cout << out << "\n";
  }
  return 0;
}
