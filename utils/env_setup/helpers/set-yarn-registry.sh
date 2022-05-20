
cd $PROGRAM_ROOT

if [ "$1" = "reset" ]
then
  # yarn config set npmRegistryServer https://$NPM_LIVE_REGISTRY
  # yarn config set unsafeHttpWhitelist --json '[]'
  cat $PROGRAM_ROOT/utils/env_setup/templates/.yarnrc.yml.live > $PROGRAM_ROOT/.yarnrc.yml
else
  # yarn config set npmRegistryServer http://$NPM_LOCAL_REGISTRY
  # yarn config set unsafeHttpWhitelist --json "[\"localhost\"]"
  cat $PROGRAM_ROOT/utils/env_setup/templates/.yarnrc.yml.dev > $PROGRAM_ROOT/.yarnrc.yml
fi
