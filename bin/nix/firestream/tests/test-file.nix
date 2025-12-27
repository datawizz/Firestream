{ pkgs, firestream }:

pkgs.runCommand "test-file" {} ''
  export HOME=$TMPDIR

  cat > $TMPDIR/test.sh << 'SCRIPT'
  ${firestream.lib.file.functions}

  # Test replace_in_file
  test_file="$TMPDIR/test.txt"
  echo "Hello PLACEHOLDER world" > "$test_file"
  replace_in_file "$test_file" "PLACEHOLDER" "beautiful"
  result=$(cat "$test_file")
  [[ "$result" == "Hello beautiful world" ]] || { echo "FAIL: replace_in_file basic replacement (got: $result)"; exit 1; }

  # Test replace_in_file with special characters
  echo "cost: $100" > "$test_file"
  replace_in_file "$test_file" '$100' '$200'
  result=$(cat "$test_file")
  [[ "$result" == "cost: \$200" ]] || { echo "FAIL: replace_in_file special chars (got: $result)"; exit 1; }

  # Test replace_in_file with regex
  echo "version=1.2.3" > "$test_file"
  replace_in_file "$test_file" 'version=[0-9.]*' 'version=2.0.0'
  result=$(cat "$test_file")
  [[ "$result" == "version=2.0.0" ]] || { echo "FAIL: replace_in_file regex (got: $result)"; exit 1; }

  # Test yml_key_set
  yml_file="$TMPDIR/test.yml"
  cat > "$yml_file" << 'YML'
  server:
    port: 8080
    host: localhost
  YML

  yml_key_set "$yml_file" "server.port" "9090"
  result=$(grep "port:" "$yml_file" | head -1)
  [[ "$result" == *"9090"* ]] || { echo "FAIL: yml_key_set (got: $result)"; exit 1; }

  # Test ini_file_set
  ini_file="$TMPDIR/test.ini"
  cat > "$ini_file" << 'INI'
  [section]
  key1=value1
  key2=value2
  INI

  ini_file_set "$ini_file" "section" "key1" "newvalue"
  result=$(grep "key1" "$ini_file")
  [[ "$result" == "key1=newvalue" ]] || { echo "FAIL: ini_file_set (got: $result)"; exit 1; }

  # Test xml_set
  xml_file="$TMPDIR/test.xml"
  cat > "$xml_file" << 'XML'
  <config>
    <setting name="timeout">30</setting>
  </config>
  XML

  xml_set "$xml_file" "//setting[@name='timeout']" "60"
  result=$(grep "timeout" "$xml_file")
  [[ "$result" == *"60"* ]] || { echo "FAIL: xml_set (got: $result)"; exit 1; }

  # Test json_set
  json_file="$TMPDIR/test.json"
  cat > "$json_file" << 'JSON'
  {
    "server": {
      "port": 8080
    }
  }
  JSON

  json_set "$json_file" ".server.port" "9090"
  result=$(cat "$json_file")
  [[ "$result" == *"9090"* ]] || { echo "FAIL: json_set (got: $result)"; exit 1; }

  # Test append_file_if_not_exists
  append_file="$TMPDIR/append.txt"
  echo "line1" > "$append_file"
  append_file_if_not_exists "$append_file" "line2"
  [[ $(wc -l < "$append_file") -eq 2 ]] || { echo "FAIL: append_file_if_not_exists should append"; exit 1; }

  # Test append_file_if_not_exists (duplicate)
  append_file_if_not_exists "$append_file" "line2"
  [[ $(wc -l < "$append_file") -eq 2 ]] || { echo "FAIL: append_file_if_not_exists should not duplicate"; exit 1; }

  # Test remove_in_file
  remove_file="$TMPDIR/remove.txt"
  cat > "$remove_file" << 'TXT'
  line1
  line2
  line3
  TXT

  remove_in_file "$remove_file" "line2"
  ! grep -q "line2" "$remove_file" || { echo "FAIL: remove_in_file should remove line"; exit 1; }
  grep -q "line1" "$remove_file" || { echo "FAIL: remove_in_file should keep other lines"; exit 1; }

  echo "All file tests passed!"
  SCRIPT

  ${pkgs.bash}/bin/bash $TMPDIR/test.sh
  touch $out
''
