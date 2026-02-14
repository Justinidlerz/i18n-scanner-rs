import { useTranslation } from 'react-i18next'
import { POST_COLLECTOR_KEY_ALIAS } from './reexport'

const PostCollectorCrossFile = () => {
  const { t } = useTranslation()

  return <p>{t(POST_COLLECTOR_KEY_ALIAS)}</p>
}

export default PostCollectorCrossFile
