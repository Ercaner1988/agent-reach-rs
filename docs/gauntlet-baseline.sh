#!/bin/bash
# Altin kume taban olcumu: her sorgu icin hedef URL ilk 10'da mi?
cd "C:/Users/buzbe/OneDrive/Masaüstü/agent-reach-rs"
BIN=./target/release/agent-reach-mcp.exe

run() { # $1=channel $2=action $3=query
  printf '%s\n' \
'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' \
'{"jsonrpc":"2.0","method":"notifications/initialized"}' \
"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}" \
  | $BIN 2>/dev/null | tail -1
}

declare -a Q=(
"a sqlite reimplementation written in rust|cursor/minisqlite"
"sqlite compatible database written in rust|tursodatabase/turso"
"control headless chrome from rust|rust-headless-chrome"
"chrome devtools protocol rust api|chromiumoxide"
"sqlite bindings for rust|rusqlite"
"webdriver client library for rust|fantoccini"
"rust http client library|reqwest"
"exa mcp server for web search|exa-mcp-server"
)

gh_hit=0; exa_hit=0; n=0
for row in "${Q[@]}"; do
  q="${row%%|*}"; want="${row##*|}"; n=$((n+1))
  g=$(run agent_reach_execute "{\"channel\":\"github\",\"action\":\"search\",\"args\":[\"$q\"]}")
  e=$(run exa_search "{\"query\":\"$q\",\"num_results\":10}")
  gok=no; eok=no
  echo "$g" | grep -qi "$want" && { gok=yes; gh_hit=$((gh_hit+1)); }
  echo "$e" | grep -qi "$want" && { eok=yes; exa_hit=$((exa_hit+1)); }
  gcount=$(echo "$g" | grep -o 'https://github.com/' | wc -l)
  printf "%-48s github:%-4s(%2d sonuc)  exa:%s\n" "$q" "$gok" "$gcount" "$eok"
done
echo "-----"
echo "github recall@10: $gh_hit/$n"
echo "exa    recall@10: $exa_hit/$n"
